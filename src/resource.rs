use lazy_static::lazy_static;
use libc::{getrusage, rusage, RUSAGE_SELF};
use std::{env, fmt, io, mem::MaybeUninit, result, time::Instant};

type Result<T> = result::Result<T, io::Error>;

lazy_static! {
    static ref NOW: Instant = Instant::now();
}

pub fn realtime() -> u64 {
    NOW.elapsed().as_secs()
}

struct Resources {
    command_line: String,
}

impl Resources {
    fn new() -> Result<Self> {
        let command_line = env::args().collect::<Vec<String>>().join(" ");

        Ok(Self { command_line })
    }

    fn get_resource_usage(&self) -> Result<rusage> {
        let r = unsafe {
            let mut r = MaybeUninit::<rusage>::uninit();
            if getrusage(RUSAGE_SELF, r.as_mut_ptr()) == -1 {
                return Err(io::Error::last_os_error());
            }
            r.assume_init()
        };
        Ok(r)
    }

    fn cputime(&self) -> Result<i64> {
        let r = self.get_resource_usage()?;
        Ok(r.ru_utime.tv_sec + r.ru_stime.tv_sec)
    }

    fn peakrss(&self) -> Result<i64> {
        let r = self.get_resource_usage()?;
        Ok(r.ru_maxrss)
    }
}

impl fmt::Display for Resources {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CMD: {}\nReal time: {} sec; CPU: {} sec; Peak RSS: {:.3} GB",
            self.command_line,
            realtime(),
            self.cputime().unwrap_or(0),
            self.peakrss().unwrap_or(0) as f64 / 1024.0 / 1024.0,
        )
    }
}

pub fn gather_resources() {
    let cmd_resources = Resources::new().unwrap();
    println!("{}", cmd_resources);
}
