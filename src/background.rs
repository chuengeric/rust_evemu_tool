/**
✅ 已实现的核心功能：

登录与角色管理：账户登录、角色选择、创建和删除等功能基本完成。
技能系统：技能训练功能已实现，但可能存在一些 Bug。
市场系统：市场的买卖功能已实现，但性能优化（如“虚拟市场播种”）是未来的工作重点。
聊天系统：大部分聊天功能已工作，但需要在 Crucible 客户端下进行更多测试。

❌ 待完善或未实现的核心系统：

舰队系统 (Fleets)：完全未实现。
模块与战斗 (Modules & Combat)：模块系统处于“非常早期”阶段，舰船战斗仅部分实现。
空间站服务 (Station Services)：大部分功能尚未正常工作。
任务与 NPC (Missions & NPCs)：完全未实现。
玩家 owned 空间站 (POSs)：完全未实现。
军团与联盟 (Corporations/Alliances)：部分工作。

*/
// src/background.rs
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use std::future::Future;
use std::pin::Pin;

// type TaskFn = Pin<Box<dyn Fn() -> Pin<Box<dyn Future<Output = String> + Send>> + Send>>;
type TaskFn = Box<dyn Fn() -> Pin<Box<dyn Future<Output = String> + Send>> + Send>;



pub struct Task {
    pub name: String,
    pub interval_secs: u64,
    pub func: TaskFn,
}

pub struct Manager {
    tasks: Vec<Task>,
    tx: mpsc::UnboundedSender<String>,
}

impl Manager {
    pub fn new(tx: mpsc::UnboundedSender<String>) -> Self {
        Self { tasks: Vec::new(), tx }
    }

    pub fn register_task<F, Fut>(&mut self, name: &str, interval_secs: u64, func: F)
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let func_boxed: TaskFn = Box::new(move || {
            let fut = func();
            Box::pin(fut)
        });
        self.tasks.push(Task {
            name: name.to_string(),
            interval_secs,
            func: func_boxed,
        });
    }

    pub async fn run(self) {
        let tx = self.tx;
        for task in self.tasks {
            let tx_clone = tx.clone();
            let interval = Duration::from_secs(task.interval_secs);
            let name = task.name.clone();
            tokio::spawn(async move {
                let mut interval_timer = time::interval(interval);
                loop {
                    interval_timer.tick().await;
                    let result = (task.func)().await;
                    let _ = tx_clone.send(format!("[后台任务: {}] {}", name, result));
                }
            });
        }
    }
}