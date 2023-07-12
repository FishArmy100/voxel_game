use std::time::SystemTime;

pub fn time_call<F, R>(f: F, name: &str) -> R
    where F : FnOnce() -> R
{
    let current_time = SystemTime::now();
    let r = f();
    let time = current_time.elapsed().unwrap().as_secs_f32() * 1000.0;
    println!("test '{}' took {}ms", name, time);
    r
}