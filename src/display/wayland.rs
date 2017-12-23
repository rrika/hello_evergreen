use std;
use super::*;
use drm_radeon_ioctl::*;

use wayland_client;
use wayland_client::{EventQueueHandle, EnvHandler};
use wayland_client::protocol::{wl_surface, wl_shm_pool, wl_buffer, wl_compositor, wl_shell,
                               wl_subcompositor, wl_shm, wl_shell_surface, wl_seat};

use wayland_protocols;
use wayland_protocols::unstable::linux_dmabuf::v1::client::zwp_linux_dmabuf_v1;
use wayland_protocols::unstable::linux_dmabuf::v1::client::zwp_linux_buffer_params_v1;

wayland_env!(WaylandEnv,
	compositor: wl_compositor::WlCompositor,
	shell: wl_shell::WlShell,
	linux_dmabuf: zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1
);

struct HelloRadeonWindow {
	s: wl_surface::WlSurface,
	display: *const wayland_client::protocol::wl_display::WlDisplay,
	buf: Option<wl_buffer::WlBuffer>,
}

unsafe impl std::marker::Send for HelloRadeonWindow {}

fn buffer_param_create__created(_: &mut EventQueueHandle,
           self_: &mut HelloRadeonWindow,
           proxy: &zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1,
           buffer: wl_buffer::WlBuffer)
{
	println!("created buf");
	self_.s.attach(Some(&buffer), 0, 0);
	self_.s.commit();
	self_.buf = Some(buffer);
	proxy.destroy();
}

fn buffer_param_create__failed(evqh: &mut EventQueueHandle,
	      self_: &mut HelloRadeonWindow,
          proxy: &zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1)
{
	
	proxy.destroy();
	print!("{:?}", unsafe{ &*self_.display }.last_error().expect("wayland buffer sharing failed but no error code"));
	panic!("wayland buffer sharing failed");
}

const buffer_param_create__implementation:
	wayland_protocols::unstable::linux_dmabuf::v1::client::zwp_linux_buffer_params_v1::Implementation<HelloRadeonWindow>
=
	wayland_protocols::unstable::linux_dmabuf::v1::client::zwp_linux_buffer_params_v1::Implementation {
		created: buffer_param_create__created,
		failed:  buffer_param_create__failed
	};


fn dmabuf_create_params__format(
          evqh: &mut EventQueueHandle,
	      self_: &mut (),
          proxy: &zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1,
          format: u32)
{
	println!("found format {}{}{}{}",
		(format as u8 as char), ((format >> 8) as u8 as char),
		((format >> 16) as u8 as char), ((format >> 24) as u8 as char));
}

const dmabuf_create_params__implementation:
	wayland_protocols::unstable::linux_dmabuf::v1::client::zwp_linux_dmabuf_v1::Implementation<()>
=
	wayland_protocols::unstable::linux_dmabuf::v1::client::zwp_linux_dmabuf_v1::Implementation {
		format: dmabuf_create_params__format,
		modifier: |_, _, _, _, _, _| {}
	};

fn shell_surface_impl() -> wl_shell_surface::Implementation<()> {
    wl_shell_surface::Implementation {
        ping: |_, _, shell_surface, serial| {
            shell_surface.pong(serial);
        },
        configure: |_, _, _, _, _, _| { /* not used in this example */ },
        popup_done: |_, _, _| { /* not used in this example */ },
    }
}

//declare_handler!(HelloRadeonWindow, wayland_protocols::unstable::linux_dmabuf::v1::client::zwp_linux_buffer_params_v1::Handler,
//	wayland_protocols::unstable::linux_dmabuf::v1::client::zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1);

//declare_handler!(HelloRadeonWindow, wayland_protocols::unstable::linux_dmabuf::v1::client::zwp_linux_dmabuf_v1::Handler,
//	wayland_protocols::unstable::linux_dmabuf::v1::client::zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1);

//declare_handler!(HelloRadeonWindow, wl_shell_surface::Handler, wl_shell_surface::WlShellSurface);


pub fn waylandmain(fd: i32, handle: u32) -> bool {
	let (display, mut event_queue) = match wayland_client::default_connect() {
		Ok(ret) => ret,
		Err(e) => return false
	};

	let display = display;

	//let envid = event_queue.add_handler(EnvHandler::<WaylandEnv>::new());
	let registry = display.get_registry(); //.expect("Display cannot be already destroyed.");
	let env_token = EnvHandler::<WaylandEnv>::init(&mut event_queue, &registry);

	// prepare the decorated surface
	let (dmabufid, window, shell_surface, buffer_params) = {

		let mut first = true;

		loop {
			event_queue.sync_roundtrip().unwrap();
			{
				// introduce a new scope because .state() borrows the event_queue
				let state = event_queue.state();
				// retrieve the EnvHandler
				let env = state.get(&env_token); //.clone_inner().unwrap();
				if first {
					println!("Globals advertised by server:");
					for &(name, ref interface, version) in env.globals() {
						println!("{:4} : {} (version {})", name, interface, version);
					}
				}
				first = false;
				if env.ready() {
					break
				}
			}
			println!("env not ready. syncing...");
		}

		let env = event_queue.state().get(&env_token).clone_inner().unwrap();
		let dmabufid = 0; //state.add_handler(&env.linux_dmabuf);

		let surface = env.compositor.create_surface(); //.expect("Compositor cannot be destroyed");
		let shell_surface = env.shell.get_shell_surface(&surface); //.expect("Shell cannot be destroyed");


		let mut ph = DrmPrimeHandle::default();
		ph.handle = handle;
		unsafe { drm_ioctl_prime_handle_to_fd(fd, &mut ph) }.unwrap();
		let cbfd = ph.fd;

		event_queue.register(&env.linux_dmabuf, dmabuf_create_params__implementation, ());
		let buffer_params = env.linux_dmabuf.create_params().expect("got no buffer params object");
		buffer_params.add(cbfd, 0, 0, W*4, 0, 0);

		// find the seat if any
		// let mut seat: Option<&wl_seat::WlSeat> = None;
		// for &(id, ref interface, _) in env.globals() {
		//     if interface == "wl_seat" {
		//         seat = Some(registry.bind(1, id).expect("Registry cannot die!"));
		//         break;
		//     }
		// }

		shell_surface.set_toplevel();
		//surface.attach(Some(&buffer), 0, 0);
		surface.commit();

		let window = HelloRadeonWindow {
			s: surface,
			display: &display,
			buf: None, //buffer,
		};

		(dmabufid, window, shell_surface, buffer_params)
	};

	//let winid = event_queue.add_handler(window);
    event_queue.register(&shell_surface, shell_surface_impl(), ());
	event_queue.register(&buffer_params, buffer_param_create__implementation, window);

	// {
	//     let state = event_queue.state();
	//     let env = state.get_handler::<EnvHandler<WaylandEnv>>(envid);
	// 	event_queue.register::<_, HelloRadeonWindow>(&env.linux_dmabuf, winid);
	// }

	let RGBX8888 = ('X' as u32) + (('R' as u32)<<8) + (('2' as u32)<<16) + (('4' as u32)<<24);
	buffer_params.create(W as i32, H as i32, RGBX8888, 0);

	loop {
		display.flush().unwrap();
		event_queue.dispatch().unwrap();
	}
}
