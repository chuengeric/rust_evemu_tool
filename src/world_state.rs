// src/world_state.rs
/**
世界状态页面设计
新页面将显示以下实时（或模拟）数据：

指标	说明
星系节点总数	当前在线星系数量（模拟全宇宙）
在线玩家分布	每个星系的玩家数（热力图）
市场订单量	当前活跃的买卖订单总数
NPC 刷新状态	各星系 NPC 刷新的最后时间
小行星带资源	各星系矿带剩余资源百分比
主权变更记录	最近的军团/联盟主权变更事件
这些数据可以通过与 EVEmu 服务器的数据库交互获得（如 mapSolarSystems、mktOrders、npcSpawn 等表
*/


// src/world_state.rs

use sqlx::{MySqlPool, Row};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use std::future::Future;
use std::pin::Pin;

// ---------- 数据结构 ----------
#[derive(Clone, Debug)]
pub struct SystemNode {
    pub id: u32,
    pub name: String,
    pub players: u32,
    pub npc_last_spawn: String,
    pub belt_remaining: f32,
    pub region: String,
}

#[derive(Clone, Debug)]
pub struct MarketOverview {
    pub total_buy_orders: u32,
    pub total_sell_orders: u32,
    pub top_traded_item: String,
}

#[derive(Clone, Debug)]
pub struct SovereigntyEvent {
    pub time: String,
    pub system: String,
    pub new_owner: String,
    pub old_owner: String,
}

// ---------- 数据源 Trait (dyn 兼容) ----------
pub trait WorldStateProvider: Send + Sync {
    fn fetch_systems(&self) -> Pin<Box<dyn Future<Output = Result<Vec<SystemNode>, String>> + Send>>;
    fn fetch_market(&self) -> Pin<Box<dyn Future<Output = Result<MarketOverview, String>> + Send>>;
    fn fetch_sovereignty_events(&self) -> Pin<Box<dyn Future<Output = Result<Vec<SovereigntyEvent>, String>> + Send>>;
}

// ---------- 模拟数据源 ----------
pub struct SimulationProvider;

impl WorldStateProvider for SimulationProvider {
    fn fetch_systems(&self) -> Pin<Box<dyn Future<Output = Result<Vec<SystemNode>, String>> + Send>> {
        Box::pin(async {
            Ok(vec![
                SystemNode { id: 30000142, name: "Jita".to_string(), players: 120, npc_last_spawn: "2026-09-08 10:00".to_string(), belt_remaining: 0.85, region: "The Forge".to_string() },
                SystemNode { id: 30002187, name: "Amarr".to_string(), players: 80, npc_last_spawn: "2026-09-08 09:45".to_string(), belt_remaining: 0.92, region: "Domain".to_string() },
                SystemNode { id: 30002659, name: "Dodixie".to_string(), players: 55, npc_last_spawn: "2026-09-08 09:30".to_string(), belt_remaining: 0.78, region: "Sinq Laison".to_string() },
            ])
        })
    }

    fn fetch_market(&self) -> Pin<Box<dyn Future<Output = Result<MarketOverview, String>> + Send>> {
        Box::pin(async {
            Ok(MarketOverview {
                total_buy_orders: 12500,
                total_sell_orders: 18000,
                top_traded_item: "Tritanium".to_string(),
            })
        })
    }

    fn fetch_sovereignty_events(&self) -> Pin<Box<dyn Future<Output = Result<Vec<SovereigntyEvent>, String>> + Send>> {
        Box::pin(async {
            Ok(vec![
                SovereigntyEvent { time: "2026-09-08 08:00".to_string(), system: "Jita".to_string(), new_owner: "Goonswarm".to_string(), old_owner: "Pandemic Legion".to_string() },
            ])
        })
    }
}

// ---------- 数据库数据源 ----------
pub struct DatabaseProvider {
    pool: Arc<MySqlPool>,
}

impl DatabaseProvider {
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self { pool }
    }
}

impl WorldStateProvider for DatabaseProvider {
    
    // solarSystems
    // fn fetch_systems(&self) -> Pin<Box<dyn Future<Output = Result<Vec<SystemNode>, String>> + Send>> {
    //     let pool = self.pool.clone();
    //     Box::pin(async move {
    //         let query = r#"
    //             SELECT solarSystemID, solarSystemName, regionID, security
    //             FROM mapSolarSystems
    //             LIMIT 100
    //         "#;
    //         let rows = sqlx::query(query)
    //             .fetch_all(&*pool)
    //             .await
    //             .map_err(|e| format!("DB error: {}", e))?;
    //         let mut systems = Vec::new();
    //         for row in rows {
    //             let id: u32 = row.get(0);
    //             let name: String = row.get(1);
    //             let region_id: u32 = row.get(2);
    //             let region = format!("Region-{}", region_id);
    //             systems.push(SystemNode {
    //                 id,
    //                 name,
    //                 players: 0,
    //                 npc_last_spawn: "Unknown".to_string(),
    //                 belt_remaining: 1.0,
    //                 region,
    //             });
    //         }
    //         Ok(systems)
    //     })
    // }
    fn fetch_systems(&self) -> Pin<Box<dyn Future<Output = Result<Vec<SystemNode>, String>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let query = r#"
                SELECT solarSystemID, solarSystemName, regionID, security
                FROM mapSolarSystems
                LIMIT 100
            "#;
            let rows = sqlx::query(query)
                .fetch_all(&*pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            let mut systems = Vec::new();
            for row in rows {
                let id: i32 = row.get(0);
                let id = id as u32;
                let name: String = row.get(1);
                let region_id: i32 = row.get(2);
                let region_id = region_id as u32;
                let region = format!("Region-{}", region_id);
                systems.push(SystemNode {
                    id,
                    name,
                    players: 0,
                    npc_last_spawn: "Unknown".to_string(),
                    belt_remaining: 1.0,
                    region,
                });
            }
            Ok(systems)
        })
    }


    // fn fetch_market(&self) -> Pin<Box<dyn Future<Output = Result<MarketOverview, String>> + Send>> {
    //     let pool = self.pool.clone();
    //     Box::pin(async move {
    //         let query = r#"
    //             SELECT
    //                 SUM(CASE WHEN bid = 1 THEN 1 ELSE 0 END) as buy_orders,
    //                 SUM(CASE WHEN bid = 0 THEN 1 ELSE 0 END) as sell_orders
    //             FROM mktOrders
    //         "#;
    //         let row = sqlx::query(query)
    //             .fetch_one(&*pool)
    //             .await
    //             .map_err(|e| format!("DB error: {}", e))?;
    //         let total_buy: i64 = row.get(0);
    //         let total_sell: i64 = row.get(1);
    //         let top_query = r#"
    //             SELECT typeID, COUNT(*) as cnt
    //             FROM mktOrders
    //             GROUP BY typeID
    //             ORDER BY cnt DESC
    //             LIMIT 1
    //         "#;
    //         let top_row = sqlx::query(top_query)
    //             .fetch_optional(&*pool)
    //             .await
    //             .map_err(|e| format!("DB error: {}", e))?;
    //         let top_item = if let Some(row) = top_row {
    //             let type_id: u32 = row.get(0);
    //             format!("TypeID-{}", type_id)
    //         } else {
    //             "Unknown".to_string()
    //         };
    //         Ok(MarketOverview {
    //             total_buy_orders: total_buy as u32,
    //             total_sell_orders: total_sell as u32,
    //             top_traded_item: top_item,
    //         })
    //     })
    // }
    fn fetch_market(&self) -> Pin<Box<dyn Future<Output = Result<MarketOverview, String>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let query = r#"
                SELECT
                    SUM(CASE WHEN bid = 1 THEN 1 ELSE 0 END) as buy_orders,
                    SUM(CASE WHEN bid = 0 THEN 1 ELSE 0 END) as sell_orders
                FROM mktOrders
            "#;
            match sqlx::query(query).fetch_one(&*pool).await {
                Ok(row) => {
                    // 尝试多种类型，避免 panic
                    let total_buy = if let Ok(v) = row.try_get::<String, _>(0) {
                        v.parse::<f64>().unwrap_or(0.0).round() as u32
                    } else if let Ok(v) = row.try_get::<f64, _>(0) {
                        v.round() as u32
                    } else if let Ok(v) = row.try_get::<i64, _>(0) {
                        v as u32
                    } else {
                        0
                    };
                    let total_sell = if let Ok(v) = row.try_get::<String, _>(1) {
                        v.parse::<f64>().unwrap_or(0.0).round() as u32
                    } else if let Ok(v) = row.try_get::<f64, _>(1) {
                        v.round() as u32
                    } else if let Ok(v) = row.try_get::<i64, _>(1) {
                        v as u32
                    } else {
                        0
                    };

                    // 热门物品（typeID 是 INT，直接获取 i32）
                    let top_query = r#"
                        SELECT typeID, COUNT(*) as cnt
                        FROM mktOrders
                        GROUP BY typeID
                        ORDER BY cnt DESC
                        LIMIT 1
                    "#;
                    // let top_item = match sqlx::query(top_query).fetch_optional(&*pool).await {
                    //     Ok(Some(row)) => {
                    //         let type_id: i32 = row.get(0);
                    //         format!("TypeID-{}", type_id)
                    //     }
                    //     _ => "Unknown".to_string(),
                    // };
                    let top_item = match sqlx::query(top_query).fetch_optional(&*pool).await {
                        Ok(Some(row)) => {
                            // 使用 u16 接收 SMALLINT UNSIGNED
                            let type_id: u16 = row.get(0);
                            format!("TypeID-{}", type_id)
                        }
                        _ => "Unknown".to_string(),
                    };

                    Ok(MarketOverview {
                        total_buy_orders: total_buy,
                        total_sell_orders: total_sell,
                        top_traded_item: top_item,
                    })
                }
                Err(e) => Err(format!("DB error: {}", e)),
            }
        })
    }

    // fn fetch_sovereignty_events(&self) -> Pin<Box<dyn Future<Output = Result<Vec<SovereigntyEvent>, String>> + Send>> {
    //     let pool = self.pool.clone();
    //     Box::pin(async move {
    //         let query = r#"
    //             SELECT solarSystemID, ownerID, lastChange
    //             FROM sovData
    //             ORDER BY lastChange DESC
    //             LIMIT 10
    //         "#;
    //         let rows = sqlx::query(query)
    //             .fetch_all(&*pool)
    //             .await
    //             .map_err(|e| format!("DB error: {}", e))?;
    //         let mut events = Vec::new();
    //         for row in rows {
    //             let system_id: u32 = row.get(0);
    //             let owner_id: u32 = row.get(1);
    //             let last_change: String = row.get(2);
    //             events.push(SovereigntyEvent {
    //                 time: last_change,
    //                 system: format!("System-{}", system_id),
    //                 new_owner: format!("Corp-{}", owner_id),
    //                 old_owner: "Unknown".to_string(),
    //             });
    //         }
    //         Ok(events)
    //     })
    // }
    fn fetch_sovereignty_events(&self) -> Pin<Box<dyn Future<Output = Result<Vec<SovereigntyEvent>, String>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let query = r#"
                SELECT solarSystemID, ownerID, lastChange
                FROM sovData
                ORDER BY lastChange DESC
                LIMIT 10
            "#;
            match sqlx::query(query).fetch_all(&*pool).await {
                Ok(rows) => {
                    let mut events = Vec::new();
                    for row in rows {
                        let system_id: i32 = row.get(0);
                        let system_id = system_id as u32;
                        let owner_id: i32 = row.get(1);
                        let owner_id = owner_id as u32;
                        let last_change: String = row.get(2);
                        events.push(SovereigntyEvent {
                            time: last_change,
                            system: format!("System-{}", system_id),
                            new_owner: format!("Corp-{}", owner_id),
                            old_owner: "Unknown".to_string(),
                        });
                    }
                    Ok(events)
                }
                Err(e) => {
                    // 表不存在或查询失败时，返回空列表，不传播错误
                    eprintln!("[WorldState] Sovereignty events unavailable: {}", e);
                    Ok(Vec::new())
                }
            }
        })
    }

}

// ---------- 世界状态管理器 ----------
pub struct WorldStateManager {
    provider: Box<dyn WorldStateProvider>,
    systems: Arc<RwLock<Vec<SystemNode>>>,
    market: Arc<RwLock<MarketOverview>>,
    events: Arc<RwLock<Vec<SovereigntyEvent>>>,
    last_update: Arc<RwLock<Option<Instant>>>,
    update_interval: Duration,
}

impl WorldStateManager {
    pub fn new(provider: Box<dyn WorldStateProvider>) -> Self {
        Self {
            provider,
            systems: Arc::new(RwLock::new(Vec::new())),
            market: Arc::new(RwLock::new(MarketOverview { total_buy_orders: 0, total_sell_orders: 0, top_traded_item: "".to_string() })),
            events: Arc::new(RwLock::new(Vec::new())),
            last_update: Arc::new(RwLock::new(None)),
            update_interval: Duration::from_secs(30),
        }
    }

    pub async fn force_update(&self) {
        let systems = self.provider.fetch_systems().await.unwrap_or_else(|e| {
            eprintln!("Failed to fetch systems: {}", e);
            Vec::new()
        });
        let market = self.provider.fetch_market().await.unwrap_or_else(|e| {
            eprintln!("Failed to fetch market: {}", e);
            MarketOverview { total_buy_orders: 0, total_sell_orders: 0, top_traded_item: "Error".to_string() }
        });
        let events = self.provider.fetch_sovereignty_events().await.unwrap_or_else(|e| {
            eprintln!("Failed to fetch events: {}", e);
            Vec::new()
        });
        *self.systems.write().await = systems;
        *self.market.write().await = market;
        *self.events.write().await = events;
        *self.last_update.write().await = Some(Instant::now());
    }

    pub async fn start_background_updater(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.update_interval);
            loop {
                interval.tick().await;
                self.force_update().await;
            }
        });
    }

    pub async fn get_systems(&self) -> Vec<SystemNode> {
        self.systems.read().await.clone()
    }
    pub async fn get_market(&self) -> MarketOverview {
        self.market.read().await.clone()
    }
    pub async fn get_events(&self) -> Vec<SovereigntyEvent> {
        self.events.read().await.clone()
    }
    pub async fn last_update_time(&self) -> Option<Instant> {
        *self.last_update.read().await
    }
}