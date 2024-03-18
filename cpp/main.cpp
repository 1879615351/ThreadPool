#include <future>
#include<iostream>
#include<atomic>
#include<mutex>
#include<functional>
#include<thread>
#include<utility>
#include<vector>
#include<queue>
#include<chrono>
#include<condition_variable>
class ThreadPool{
public:
    ThreadPool(int n){
        threads.resize(n);
        for(auto &t:threads){
            t = std::thread(&ThreadPool::run,this);
        };
    }
    ~ThreadPool();
    ThreadPool(ThreadPool const &)=delete;
    ThreadPool(ThreadPool  &&)=delete;
    ThreadPool& operator= (ThreadPool const &)=delete;
    template<typename F,typename... Args>
    void addTask(F&& f,Args&&... args);
private:
    void run();
    std::vector<std::thread> threads;
    std::queue<std::function<void(void)>> tasks;
    std::atomic<bool> stop {false};
    std::mutex m_mutex;
    std::condition_variable condition;
};
void ThreadPool::run(){
    while(true){
        std::unique_lock<std::mutex> locker(m_mutex);
        condition.wait(locker,[this](){
            return !tasks.empty() || stop.load();
        });
        if(stop.load()){
            return ;
        }
        //任务队列有任务且不退出
        std::function<void(void)> f(std::move(tasks.front()));
        tasks.pop();
        locker.unlock();
        //f();
        auto promise = std::async(std::launch::async,f); //异步执行
        promise.wait();
    }
}
template<typename F,typename... Args>
void ThreadPool::addTask(F&& f,Args&&...  args){
    if(stop.load()){
        return;
    }
    std::function<void(void)> task = std::bind(std::forward<F>(f),std::forward<Args>(args)...);
    std::lock_guard<std::mutex> locker(m_mutex);
    tasks.emplace(std::move(task));
    condition.notify_one();
}
ThreadPool::~ThreadPool(){
    stop.store(true);
    condition.notify_all();
    for(auto &t:threads){
        t.join();
    }
}
int main(){
    ThreadPool pool(20);
    for(int i = 0;i<50;i++){
        pool.addTask([](int a,double b,char c){
            std::cout<<"the "<<a<<"task begin"<<std::endl;
            std::this_thread::sleep_for(std::chrono::seconds(2));
            std::cout<<"the "<<a<<"task end"<<std::endl;
        },i,2.0,i+'0');
        //std::this_thread::sleep_for(std::chrono::milliseconds(500));
    }
    while(true){
        std::cout<<"CCCC"<<std::endl;
        std::this_thread::sleep_for(std::chrono::seconds(2));
    }
    return 0;
}