// #![windows_subsystem = "windows"]

extern crate sdl3;

use oxidase::{Allocator, SourceType};
use rquickjs::prelude::MutFn;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use std::fs::read_to_string;
use std::path::Path;
use std::time::Duration;

pub fn main() {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let isolate = &mut v8::Isolate::new(Default::default());

    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope, Default::default());
    let scope = &mut v8::ContextScope::new(scope, context);

    //This function will be exposed to calling from javascript and will be on global object
    let first_function = v8::Function::new(
        scope,
        |scope: &mut v8::HandleScope,
         args: v8::FunctionCallbackArguments,
         mut rv: v8::ReturnValue| {
            let r = args.get(1).int32_value(scope).unwrap();
            println!("number: {}", r);

            let arg = args.get(0);
            let arg_string = arg.to_string(scope).unwrap().to_rust_string_lossy(scope);
            println!("{}", arg_string);
            let returned_value_string =
                v8::String::new(scope, "This is returned from rust to javascript")
                    .unwrap()
                    .into();
            rv.set(returned_value_string);
        },
    )
    .unwrap()
    .into();

    //Name of function which be used in javascript
    let name = v8::String::new(scope, "testFunction").unwrap().into();

    //Global javascript object
    let global = context.global(scope);

    //Set my function to global javascript object
    global.set(scope, name, first_function);

    let code = v8::String::new(scope, "testFunction('abc' ,1234*2)").unwrap();
    let script = v8::Script::compile(scope, code, None).unwrap();
    let result = script.run(scope).unwrap();
    let result = result.to_string(scope).unwrap();
    println!("result: {}", result.to_rust_string_lossy(scope));

    let runtime = rquickjs::Runtime::new().unwrap();
    let context = rquickjs::Context::full(&runtime).unwrap();

    let mut sbuf = read_to_string(Path::new("foo.ts")).unwrap();
    let alloc = Allocator::default();
    let _ret = oxidase::transpile(&alloc, SourceType::ts(), &mut sbuf);

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
        let _ = ctx.globals().set(
            "drawcolor",
            rquickjs::Function::new(
                ctx.clone(),
                MutFn::new(move |r: u8, g: u8, b: u8| {
                    canvas.set_draw_color(Color::RGB(r, g, b));
                    canvas.clear();
                    canvas.present();
                }),
            )
            .unwrap()
            .with_name("drawcolor"),
        );
        ctx.eval::<(), _>(sbuf).unwrap();
    });

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode, .. } => {
                    match keycode {
                        Some(Keycode::Escape) => break 'running,
                        _ => {
                            println!("hey {}", keycode.unwrap());
                        }
                    };
                }
                _ => {}
            }
        }

        // The rest of the game loop goes here...

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
