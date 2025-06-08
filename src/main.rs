#![windows_subsystem = "windows"]

extern crate sdl3;

use oxidase::{Allocator, SourceType};
use rquickjs::prelude::MutFn;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use std::time::Duration;

pub fn main() {
    let runtime = rquickjs::Runtime::new().unwrap();
    let context = rquickjs::Context::full(&runtime).unwrap();

    let mut sbuf = "drawcolor(128 as number, 0, 0)".to_string();
    let alloc = Allocator::default();
    let _ret = oxidase::transpile(&alloc, SourceType::ts(), &mut sbuf);
    println!("{}", sbuf);

    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Mini Game Maker", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();
    let mut event_pump = sdl_context.event_pump().unwrap();

    context.with(|ctx| {
        let f = MutFn::new(move |r: u8, g: u8, b: u8| {
            canvas.set_draw_color(Color::RGB(r, g, b));
            canvas.clear();
            canvas.present();
        });

        let f = rquickjs::Function::new(ctx.clone(), f)
            .unwrap()
            .with_name("hi");
        let _ = ctx.globals().set("drawcolor", f);
        ctx.eval::<(), _>(sbuf).unwrap();
    });

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // The rest of the game loop goes here...

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
