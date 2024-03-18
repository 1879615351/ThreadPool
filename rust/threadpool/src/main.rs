use std::collections::{VecDeque};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool,Ordering};
use std::sync::{Arc,Mutex};
use std::thread::sleep;
use async_std::task;
struct ThreadPool{
    threads : Vec<std::thread::JoinHandle<()>>,
    stop_flag    : Arc<AtomicBool>,
    tasks   : Arc<Mutex<VecDeque<Box::<dyn FnOnce() -> () + Send>>>>,
    num     : usize
}
impl ThreadPool{
    fn new (num :usize)->ThreadPool{
        let pool = ThreadPool{
            threads: Vec::new(),
            stop_flag :   Arc::new(AtomicBool::new(false)),
            tasks:   Arc::new(Mutex::new(VecDeque::new())),
            num:num
        };
        pool
    }
    fn start(self: &mut Self){
        for i in 0..self.num{
            let tasks_cloned = self.tasks.clone();
            let stop_flag  = self.stop_flag.clone();
            let t = std::thread::spawn(move ||{
                println!("thread start");
                loop{
                        if let Ok(mut task_lock) = tasks_cloned.lock(){
                            if !task_lock.is_empty() || stop_flag.load(Ordering::SeqCst){  //任务队列不为空或者退出线程池
                                if stop_flag.load(Ordering::SeqCst){  //退出线程池
                                    return;
                                }else{ //处理任务
                                    if let Some(f) = task_lock.pop_front(){
                                        let _ = task::spawn(async move{
                                            f();    //异步执行
                                        });
                                        //同步执行需要放在锁外面，否则枷加锁影响效率
                                        //f();
                                    }
                                }
                            }
                        }
                    } //锁自动释放
                });
            self.threads.push(t);
        }
    }
    fn addTasks<F>(self:&Self,f :F)
    where 
        F:FnOnce() -> () + Send + 'static
    {
        if let Ok(mut task_lock) = self.tasks.lock(){
            task_lock.push_back(Box::new(f));
        }
    }
    fn stop(self:&mut Self){
        self.stop_flag.store(true, Ordering::SeqCst);
        while self.threads.len()!=0{
            if let Some(t) = self.threads.pop(){
                t.join();
            }
        }
    }   
}
fn main() {
    println!("1111");
    let mut pool  = ThreadPool::new(20);
    pool.start();
    println!("2222");
    for i in 0..50{
        pool.addTasks(move||{
            println!("the {} tasks" ,i);
            std::thread::sleep(std::time::Duration::from_secs(2));
            println!("the {} tasks end" ,i);
        });
        //println!("3333");
        //std::thread::sleep(std::time::Duration::from_secs(1));
    }
    println!("Hello, world!");
    loop{
        println!("main loop");
        //pool.stop();
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
