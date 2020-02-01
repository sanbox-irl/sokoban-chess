use super::{Input, Vec2};
use winit::{
    dpi::LogicalPosition,
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput as WinitKeyboardInput,
        MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    platform::desktop::EventLoopExtDesktop,
    window::Window as WinitWindow,
};

pub fn poll_events<F>(
    input_component: &mut Input,
    events_loop: &mut EventLoop<()>,
    winit_window: &WinitWindow,
    mut callback: F,
) where
    F: FnMut(&Event<'_, ()>, &WinitWindow),
{
    let keys_pressed_last_frame = input_component.kb_input.pressed_keys.clone();
    let mouse_button_clicked_last_frame = input_component.mouse_input.mouse_pressed;
    input_component.clear_input();

    events_loop.run_return(|event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Callback
        callback(&event, winit_window);

        // Our own Input Handling
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                input_component.end_requested = true;
                *control_flow = ControlFlow::Exit;
            }

            Event::MainEventsCleared => {
                *control_flow = ControlFlow::Exit;
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(logical),
                ..
            } => {
                input_component.new_frame_size = Some(Vec2::new(logical.width as f32, logical.height as f32));
                info!("New Requested Frame Size: {:?}", input_component.new_frame_size);
            }

            Event::DeviceEvent {
                event:
                    DeviceEvent::Key(WinitKeyboardInput {
                        virtual_keycode: Some(code),
                        state,
                        ..
                    }),
                ..
            } => input_component.record_input(state, code, &keys_pressed_last_frame),

            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                input_component.mouse_input.mouse_position = Vec2::new(position.x as f32, position.y as f32);
            }

            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button,
                        ..
                    },
                ..
            } => {
                let this_button = match button {
                    WinitMouseButton::Left => 0,
                    WinitMouseButton::Right => 1,
                    WinitMouseButton::Middle => 2,
                    WinitMouseButton::Other(num) => num as usize,
                };

                if mouse_button_clicked_last_frame[this_button] == false {
                    input_component.mouse_input.mouse_pressed[this_button] = true;
                    input_component.mouse_input.mouse_held[this_button] = true;
                }
            }

            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Released,
                        button,
                        ..
                    },
                ..
            } => {
                let this_button = match button {
                    WinitMouseButton::Left => 0,
                    WinitMouseButton::Right => 1,
                    WinitMouseButton::Middle => 2,
                    WinitMouseButton::Other(num) => num as usize,
                };

                if input_component.mouse_input.mouse_pressed[this_button]
                    || input_component.mouse_input.mouse_held[this_button]
                {
                    input_component.mouse_input.mouse_pressed[this_button] = false;
                    input_component.mouse_input.mouse_held[this_button] = false;

                    input_component.mouse_input.mouse_released[this_button] = true;
                }
            }

            Event::WindowEvent {
                event: WindowEvent::MouseWheel {
                    delta: scroll_delta, ..
                },
                ..
            } => match scroll_delta {
                MouseScrollDelta::PixelDelta(LogicalPosition {
                    x: _,
                    y: vertical_move,
                }) => {
                    input_component.mouse_input.mouse_vertical_scroll_delta = -vertical_move as f32;
                }

                MouseScrollDelta::LineDelta(_, vertical_move) => {
                    input_component.mouse_input.mouse_vertical_scroll_delta = -vertical_move;
                }
            },

            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            WinitKeyboardInput {
                                state,
                                virtual_keycode: Some(code),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if cfg!(feature = "metal") {
                    input_component.record_input(state, code, &keys_pressed_last_frame);
                }
            }

            _ => {}
        }
    });
}
