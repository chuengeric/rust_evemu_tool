// fn main() {
//     println!("Hello, world!");
// }


// add textinput
/**
功能完整的 EVEmu 管理工具，包含：

账户管理：列表、添加、编辑（修改密码和角色）、删除
角色管理：按账户筛选、列表、添加、删除
物品管理：按角色/账户查询，列表，删除（简化）
SQL 控制台：执行 SQL 脚本，显示结果
服务器状态：显示在线玩家数、运行时间等
日志查看：实时查看服务器日志

 $env:LIBRARY_PATH = "C:\w64devkit\mingw64\lib"
 $env:RUSTFLAGS = "-C link-args=-lmcfgthread"
 cargo build
*/








use tokio::process::Command as TokioCommand;
use eframe::{egui, Frame};
use egui::{Button, CentralPanel, Context, ScrollArea, SidePanel, TextEdit, Window};
use sqlx::{Column, MySqlPool, Row};
use std::sync::{Arc, mpsc};
use std::time::SystemTime;
use std::process::{Command};
use std::io::Write;


mod background;

// ---------- 页面枚举 ----------
#[derive(PartialEq)]
enum Page {
    Dashboard,
    Accounts,
    Characters,
    Items,
    SqlConsole,
    ServerStatus,
    LogViewer,
    NetworkMonitor,
    ProcessControl,
    LivePlayers,
    Broadcast,
    ConfigEditor,
    BackupManager,
    Help,
    // Extend MOD
    Fleets,
    Combat,
    StationServices,
    Missions,
    POS,
    Corporations,
}

// ---------- 数据结构 ----------
#[derive(Clone)]
struct Account {
    id: u32,
    name: String,
    role: u64,
    password: String,
}

#[derive(Clone)]
struct AccountEdit {
    id: u32,
    name: String,
    password: String,
    role: String,
}

#[derive(Clone)]
struct Character {
    id: u32,
    name: String,
    account_id: u32,
}

#[derive(Clone)]
struct Item {
    id: u32,
    type_id: u32,
    owner_id: u32,
    quantity: i32,
    location_id: u32,
}

#[derive(Default)]
struct ServerStatus {
    online_players: u32,
    uptime_seconds: u64,
    last_update: Option<SystemTime>,

}

#[derive(Clone)]
struct ConnectionInfo {
    src_ip: String,
    src_port: u16,
    dst_ip: String,
    dst_port: u16,
    state: String,
}

// 新增：实时玩家结构
#[derive(Clone)]
struct LivePlayer {
    account_name: String,
    character_name: String,
    online_since: Option<SystemTime>,
}

// 新增：未实现模块
#[derive(Clone)]
struct Fleet {
    id: u32,
    name: String,
    members: u32,
    system: String,
}

#[derive(Clone)]
struct CombatLog {
    id: u32,
    time: String,
    ship: String,
    damage: u32,
    target: String,
}

#[derive(Clone)]
struct StationService {
    id: u32,
    name: String,
    station: String,
    status: String,
}

#[derive(Clone)]
struct Mission {
    id: u32,
    name: String,
    agent: String,
    status: String,
}

#[derive(Clone)]
struct POS {
    id: u32,
    name: String,
    system: String,
    state: String,
}

#[derive(Clone)]
struct Corporation {
    id: u32,
    name: String,
    ceo: String,
    members: u32,
}

// ---------- 主应用 ----------
struct MyApp {
    pool: Arc<MySqlPool>,
    current_page: Page,

    // Accounts
    accounts: Vec<Account>,
    editing_account: Option<AccountEdit>,
    new_account_name: String,
    new_account_pass: String,
    new_account_role: String,

    // Characters
    characters: Vec<Character>,
    selected_account_for_char: Option<u32>,
    new_char_name: String,

    // Items
    items: Vec<Item>,
    filter_owner: String,

    // SQL Console
    sql_input: String,
    sql_output: String,

    // Server Status
    server_status: ServerStatus,

    // Log Viewer
    log_content: String,
    log_auto_refresh: bool,

    // Network Monitor
    connections: Vec<ConnectionInfo>,

    // Process Control
    server_process_running: bool,
    server_pid: Option<u32>,
    server_stdin: Option<std::process::ChildStdin>, // 用于广播

    // Live Players
    live_players: Vec<LivePlayer>,

    // Broadcast
    broadcast_message: String,

    // Config Editor
    config_content: String,
    config_path: String,
    config_modified: bool,

    // Backup Manager
    backup_path: String,
    backup_status: String,

    // 扩展状态（不修改 ServerStatus）
    server_version: String,
    client_build: u32,
    static_load_ms: f64,
    db_connected: bool,
    tcp_started: bool,
    is_online: bool,

    // help command line
    help_commands: Vec<(String, String)>,

    // Extend Mods
    fleets: Vec<Fleet>,
    combat_logs: Vec<CombatLog>,
    station_services: Vec<StationService>,
    missions: Vec<Mission>,
    pos_list: Vec<POS>,
    corporations: Vec<Corporation>,

    // Channels
    tx_accounts: mpsc::Sender<Vec<Account>>,
    rx_accounts: mpsc::Receiver<Vec<Account>>,
    tx_chars: mpsc::Sender<Vec<Character>>,
    rx_chars: mpsc::Receiver<Vec<Character>>,
    tx_items: mpsc::Sender<Vec<Item>>,
    rx_items: mpsc::Receiver<Vec<Item>>,
    tx_status: mpsc::Sender<ServerStatus>,
    rx_status: mpsc::Receiver<ServerStatus>,
    tx_log: mpsc::Sender<String>,
    rx_log: mpsc::Receiver<String>,
    tx_sql: mpsc::Sender<String>,
    rx_sql: mpsc::Receiver<String>,
    tx_connections: mpsc::Sender<Vec<ConnectionInfo>>,
    rx_connections: mpsc::Receiver<Vec<ConnectionInfo>>,
    tx_live_players: mpsc::Sender<Vec<LivePlayer>>,
    rx_live_players: mpsc::Receiver<Vec<LivePlayer>>,
}

impl MyApp {
    fn new(pool: Arc<MySqlPool>) -> Self {
        let (tx_acc, rx_acc) = mpsc::channel();
        let (tx_ch, rx_ch) = mpsc::channel();
        let (tx_it, rx_it) = mpsc::channel();
        let (tx_st, rx_st) = mpsc::channel();
        let (tx_lo, rx_lo) = mpsc::channel();
        let (tx_sq, rx_sq) = mpsc::channel();
        let (tx_cn, rx_cn) = mpsc::channel();
        let (tx_lp, rx_lp) = mpsc::channel();

        Self {
            pool,
            current_page: Page::Dashboard,
            accounts: Vec::new(),
            editing_account: None,
            new_account_name: String::new(),
            new_account_pass: String::new(),
            new_account_role: String::from("7131450020691447808"),
            characters: Vec::new(),
            selected_account_for_char: None,
            new_char_name: String::new(),
            items: Vec::new(),
            filter_owner: String::new(),
            sql_input: String::new(),
            sql_output: String::new(),
            server_status: ServerStatus::default(),
            log_content: String::new(),
            log_auto_refresh: true,
            connections: Vec::new(),
            server_process_running: false,
            server_pid: None,
            server_stdin: None,
            live_players: Vec::new(),
            broadcast_message: String::new(),
            config_content: String::new(),
            config_path: String::from("../etc/eve-server.xml"),
            config_modified: false,
            backup_path: String::from("../backup/"),
            backup_status: String::new(),
            tx_accounts: tx_acc,
            rx_accounts: rx_acc,
            tx_chars: tx_ch,
            rx_chars: rx_ch,
            tx_items: tx_it,
            rx_items: rx_it,
            tx_status: tx_st,
            rx_status: rx_st,
            tx_log: tx_lo,
            rx_log: rx_lo,
            tx_sql: tx_sq,
            rx_sql: rx_sq,
            tx_connections: tx_cn,
            rx_connections: rx_cn,
            tx_live_players: tx_lp,
            rx_live_players: rx_lp,
            
            // 
            server_version: String::new(),
            client_build: 0,
            static_load_ms: 0.0,
            db_connected: false,
            tcp_started: false,
            is_online: false,

            // 
            help_commands: Vec::new(),
            // extend mods
            fleets: Vec::new(),
            combat_logs: Vec::new(),
            station_services: Vec::new(),
            missions: Vec::new(),
            pos_list: Vec::new(),
            corporations: Vec::new(),
        }
    }

    // ---------- 账户操作 ----------
    fn load_accounts(&mut self) {
        let pool = self.pool.clone();
        let tx = self.tx_accounts.clone();
        tokio::spawn(async move {
            let rows = sqlx::query(
                "SELECT accountID, accountName, role, password FROM account",
            )
            .fetch_all(&*pool)
            .await;
            match rows {
                Ok(rows) => {
                    let accounts: Vec<Account> = rows
                        .iter()
                        .map(|row| {
                            let id: u32 = row.get(0);
                            let name: String = row.get(1);
                            let role: u64 = row.get(2);
                            let password: String = row.get(3);
                            Account { id, name, role, password }
                        })
                        .collect();
                    let _ = tx.send(accounts);
                }
                Err(e) => eprintln!("[ERROR] Load accounts: {}", e),
            }
        });
    }

    fn add_account(&mut self) {
        if self.new_account_name.is_empty() || self.new_account_pass.is_empty() {
            return;
        }
        let pool = self.pool.clone();
        let name = self.new_account_name.clone();
        let password = self.new_account_pass.clone();
        let role = self.new_account_role.parse::<u64>().unwrap_or(7131450020691447808);
        let tx = self.tx_accounts.clone();
        tokio::spawn(async move {
            if let Err(e) = sqlx::query(
                "INSERT INTO account (accountName, password, role) VALUES (?, ?, ?)",
            )
            .bind(&name)
            .bind(&password)
            .bind(role)
            .execute(&*pool)
            .await
            {
                eprintln!("[ERROR] Add account: {}", e);
                return;
            }
            if let Ok(rows) = sqlx::query(
                "SELECT accountID, accountName, role, password FROM account",
            )
            .fetch_all(&*pool)
            .await
            {
                let accounts: Vec<Account> = rows
                    .iter()
                    .map(|row| {
                        let id: u32 = row.get(0);
                        let name: String = row.get(1);
                        let role: u64 = row.get(2);
                        let password: String = row.get(3);
                        Account { id, name, role, password }
                    })
                    .collect();
                let _ = tx.send(accounts);
            }
        });
        self.new_account_name.clear();
        self.new_account_pass.clear();
    }

    fn delete_account(&mut self, id: u32) {
        let pool = self.pool.clone();
        let tx = self.tx_accounts.clone();
        tokio::spawn(async move {
            if let Err(e) = sqlx::query("DELETE FROM account WHERE accountID = ?")
                .bind(id)
                .execute(&*pool)
                .await
            {
                eprintln!("[ERROR] Delete account: {}", e);
                return;
            }
            if let Ok(rows) = sqlx::query(
                "SELECT accountID, accountName, role, password FROM account",
            )
            .fetch_all(&*pool)
            .await
            {
                let accounts: Vec<Account> = rows
                    .iter()
                    .map(|row| {
                        let id: u32 = row.get(0);
                        let name: String = row.get(1);
                        let role: u64 = row.get(2);
                        let password: String = row.get(3);
                        Account { id, name, role, password }
                    })
                    .collect();
                let _ = tx.send(accounts);
            }
        });
    }

    fn update_account(&mut self, edit: AccountEdit) {
        let pool = self.pool.clone();
        let tx = self.tx_accounts.clone();
        tokio::spawn(async move {
            if let Err(e) = sqlx::query(
                "UPDATE account SET accountName = ?, password = ?, role = ? WHERE accountID = ?",
            )
            .bind(&edit.name)
            .bind(&edit.password)
            .bind(edit.role.parse::<u64>().unwrap_or(7131450020691447808))
            .bind(edit.id)
            .execute(&*pool)
            .await
            {
                eprintln!("[ERROR] Update account: {}", e);
                return;
            }
            if let Ok(rows) = sqlx::query(
                "SELECT accountID, accountName, role, password FROM account",
            )
            .fetch_all(&*pool)
            .await
            {
                let accounts: Vec<Account> = rows
                    .iter()
                    .map(|row| {
                        let id: u32 = row.get(0);
                        let name: String = row.get(1);
                        let role: u64 = row.get(2);
                        let password: String = row.get(3);
                        Account { id, name, role, password }
                    })
                    .collect();
                let _ = tx.send(accounts);
            }
        });
    }

    // ---------- 角色操作 ----------
    fn load_characters(&mut self, account_id: u32) {
        let pool = self.pool.clone();
        let tx = self.tx_chars.clone();
        tokio::spawn(async move {
            let rows = sqlx::query(
                "SELECT characterID, characterName FROM chrCharacters WHERE accountID = ?",
            )
            .bind(account_id)
            .fetch_all(&*pool)
            .await;
            match rows {
                Ok(rows) => {
                    let chars = rows
                        .iter()
                        .map(|row| {
                            let id: u32 = row.get(0);
                            let name: String = row.get(1);
                            Character { id, name, account_id }
                        })
                        .collect();
                    let _ = tx.send(chars);
                }
                Err(e) => eprintln!("[ERROR] Load characters: {}", e),
            }
        });
    }

    fn add_character(&mut self, account_id: u32) {
        if self.new_char_name.is_empty() {
            return;
        }
        let pool = self.pool.clone();
        let name = self.new_char_name.clone();
        let tx = self.tx_chars.clone();
        tokio::spawn(async move {
            if let Err(e) = sqlx::query(
                "INSERT INTO chrCharacters (characterName, accountID) VALUES (?, ?)",
            )
            .bind(&name)
            .bind(account_id)
            .execute(&*pool)
            .await
            {
                eprintln!("[ERROR] Add character: {}", e);
                return;
            }
            if let Ok(rows) = sqlx::query(
                "SELECT characterID, characterName FROM chrCharacters WHERE accountID = ?",
            )
            .bind(account_id)
            .fetch_all(&*pool)
            .await
            {
                let chars = rows
                    .iter()
                    .map(|row| {
                        let id: u32 = row.get(0);
                        let name: String = row.get(1);
                        Character { id, name, account_id }
                    })
                    .collect();
                let _ = tx.send(chars);
            }
        });
        self.new_char_name.clear();
    }

    fn delete_character(&mut self, id: u32, account_id: u32) {
        let pool = self.pool.clone();
        let tx = self.tx_chars.clone();
        tokio::spawn(async move {
            if let Err(e) = sqlx::query("DELETE FROM chrCharacters WHERE characterID = ?")
                .bind(id)
                .execute(&*pool)
            .await
            {
                eprintln!("[ERROR] Delete character: {}", e);
                return;
            }
            if let Ok(rows) = sqlx::query(
                "SELECT characterID, characterName FROM chrCharacters WHERE accountID = ?",
            )
            .bind(account_id)
            .fetch_all(&*pool)
            .await
            {
                let chars = rows
                    .iter()
                    .map(|row| {
                        let id: u32 = row.get(0);
                        let name: String = row.get(1);
                        Character { id, name, account_id }
                    })
                    .collect();
                let _ = tx.send(chars);
            }
        });
    }

    // ---------- 物品操作 ----------
    fn load_items(&mut self, owner_id: Option<u32>) {
        let pool = self.pool.clone();
        let tx = self.tx_items.clone();
        tokio::spawn(async move {
            let mut query = String::from(
                "SELECT itemID, typeID, ownerID, quantity, locationID FROM entity WHERE 1=1",
            );
            let mut bind = Vec::new();
            if let Some(oid) = owner_id {
                query.push_str(" AND ownerID = ?");
                bind.push(oid);
            }
            let mut q = sqlx::query(&query);
            for b in bind {
                q = q.bind(b);
            }
            let rows = q.fetch_all(&*pool).await;
            match rows {
                Ok(rows) => {
                    let items = rows
                        .iter()
                        .map(|row| {
                            let id: u32 = row.get(0);
                            let type_id: u32 = row.get(1);
                            let owner_id: u32 = row.get(2);
                            let quantity: i32 = row.get(3);
                            let location_id: u32 = row.get(4);
                            Item { id, type_id, owner_id, quantity, location_id }
                        })
                        .collect();
                    let _ = tx.send(items);
                }
                Err(e) => eprintln!("[ERROR] Load items: {}", e),
            }
        });
    }

    // ---------- 服务器状态 ----------
    fn load_server_status(&mut self) {
        let pool = self.pool.clone();
        let tx = self.tx_status.clone();
        tokio::spawn(async move {
            let online = sqlx::query("SELECT COUNT(*) FROM account WHERE online = 1")
                .fetch_one(&*pool)
                .await
                .map(|row| row.get::<i64, _>(0) as u32)
                .unwrap_or(0);
            let status = ServerStatus {
                online_players: online,
                uptime_seconds: 0,
                last_update: Some(SystemTime::now()),
            };
            let _ = tx.send(status);
        });
    }

    // ---------- SQL 控制台 ----------
    fn execute_sql(&mut self) {
        if self.sql_input.trim().is_empty() {
            return;
        }
        let pool = self.pool.clone();
        let sql = self.sql_input.clone();
        let tx = self.tx_sql.clone();
        tokio::spawn(async move {
            let is_select = sql.trim().to_lowercase().starts_with("select");
            if is_select {
                let rows = sqlx::query(&sql).fetch_all(&*pool).await;
                match rows {
                    Ok(rows) => {
                        let mut output = String::new();
                        if rows.is_empty() {
                            output = "No rows returned.".to_string();
                        } else {
                            let columns = rows[0].columns();
                            let col_names: Vec<String> = columns.iter().map(|c| c.name().to_string()).collect();
                            output.push_str(&col_names.join("\t"));
                            output.push('\n');
                            for row in &rows {
                                let row_str: Vec<String> = columns.iter().map(|c| {
                                    let col_name = c.name();
                                    row.try_get::<String, _>(col_name).unwrap_or_else(|_| "NULL".to_string())
                                }).collect();
                                output.push_str(&row_str.join("\t"));
                                output.push('\n');
                            }
                        }
                        let _ = tx.send(output);
                    }
                    Err(e) => {
                        let _ = tx.send(format!("Error: {}", e));
                    }
                }
            } else {
                let res = sqlx::query(&sql).execute(&*pool).await;
                match res {
                    Ok(r) => {
                        let msg = format!("Rows affected: {}", r.rows_affected());
                        let _ = tx.send(msg);
                    }
                    Err(e) => {
                        let _ = tx.send(format!("Error: {}", e));
                    }
                }
            }
        });
    }

    // ---------- 日志查看 ----------
    // fn load_log(&mut self) {
    //     let path = std::path::Path::new("../logs/eve-server.log");
    //     if path.exists() {
    //         if let Ok(content) = std::fs::read_to_string(path) {
    //             let _ = self.tx_log.send(content);
    //         }
    //     } else {
    //         let _ = self.tx_log.send("Log file not found.".to_string());
    //     }
    // }
    fn load_log(&mut self) {
        let log_dir = std::path::Path::new("../logs/"); // 根据实际路径调整
        if !log_dir.exists() {
            let _ = self.tx_log.send("Log directory not found.".to_string());
            return;
        }

        // 获取所有 .log 文件，按修改时间排序，取最新的
        let entries = std::fs::read_dir(log_dir);
        if let Ok(entries) = entries {
            let mut log_files: Vec<_> = entries
                .filter_map(Result::ok)
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "log"))
                .collect();
            if log_files.is_empty() {
                let _ = self.tx_log.send("No log files found.".to_string());
                return;
            }
            // 按修改时间排序（最新的在前）
            log_files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
            if let Some(latest) = log_files.last() {
                if let Ok(content) = std::fs::read_to_string(latest.path()) {
                    let _ = self.tx_log.send(content);
                } else {
                    let _ = self.tx_log.send("Failed to read log file.".to_string());
                }
            } else {
                let _ = self.tx_log.send("No log files found.".to_string());
            }
        } else {
            let _ = self.tx_log.send("Failed to read log directory.".to_string());
        }
    }

    // ---------- 网络监控 ----------
    fn refresh_connections(&mut self) {
        let tx = self.tx_connections.clone();
        tokio::task::spawn_blocking(move || {
            let mut conns = Vec::new();
            #[cfg(windows)]
            {
                if let Ok(out) = Command::new("netstat")
                    .args(&["-an"])
                    .output()
                {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    for line in stdout.lines() {
                        if !line.contains("26000") && !line.contains("26001") {
                            continue;
                        }
                        if line.contains("ESTABLISHED") || line.contains("TIME_WAIT") || line.contains("CLOSE_WAIT") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 4 && parts[0] == "TCP" {
                                let local = parts[1];
                                let remote = parts[2];
                                let state = parts[3];
                                if let (Some((src_ip, src_port)), Some((dst_ip, dst_port))) =
                                    (parse_ip_port(local), parse_ip_port(remote))
                                {
                                    conns.push(ConnectionInfo {
                                        src_ip,
                                        src_port,
                                        dst_ip,
                                        dst_port,
                                        state: state.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            #[cfg(unix)]
            {
                // Linux 可使用 `ss -tuan` 或 `netstat -tuan`
            }
            let _ = tx.send(conns);
        });
    }

    // ---------- 进程控制 ----------
    fn check_server_status(&mut self) {
        #[cfg(windows)]
        {
            if let Ok(out) = Command::new("tasklist")
                .args(&["/FI", "IMAGENAME eq eve-server.exe"])
                .output()
            {
                let out_str = String::from_utf8_lossy(&out.stdout);
                if out_str.contains("eve-server.exe") {
                    self.server_process_running = true;
                    for line in out_str.lines() {
                        if line.contains("eve-server.exe") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                if let Ok(pid) = parts[1].parse::<u32>() {
                                    self.server_pid = Some(pid);
                                }
                            }
                            break;
                        }
                    }
                } else {
                    self.server_process_running = false;
                    self.server_pid = None;
                }
            } else {
                self.server_process_running = false;
                self.server_pid = None;
            }
        }
        #[cfg(unix)]
        {
            if let Ok(out) = Command::new("pgrep").arg("eve-server").output() {
                if out.status.success() {
                    self.server_process_running = true;
                    if let Ok(pid_str) = String::from_utf8(out.stdout) {
                        if let Ok(pid) = pid_str.trim().parse::<u32>() {
                            self.server_pid = Some(pid);
                        }
                    }
                } else {
                    self.server_process_running = false;
                    self.server_pid = None;
                }
            } else {
                self.server_process_running = false;
                self.server_pid = None;
            }
        }
    }




    /**
    功能	实现方式
    实时启动日志	捕获 eve-server.exe 的 stdout/stderr，在 UI 中滚动显示
    服务器元信息	解析日志中的版本、构建日期、客户端构建号等
    启动耗时统计	解析 Static Data loaded in Xms 等时间信息
    模块状态指示	显示数据库连接、TCP 服务、各管理器状态
    */
    // fn start_server(&mut self) {
    //     if self.server_process_running {
    //         return;
    //     }
    //     let exe_path = std::path::Path::new("../src/eve-server/eve-server.exe");
    //     if !exe_path.exists() {
    //         eprintln!("eve-server.exe not found at {:?}", exe_path);
    //         return;
    //     }
    //     match Command::new(exe_path)
    //         .stdin(Stdio::piped())
    //         .spawn()
    //     {
    //         Ok(mut child) => {
    //             self.server_process_running = true;
    //             self.server_pid = Some(child.id());
    //             // 保存 stdin 句柄以便发送广播
    //             if let Some(stdin) = child.stdin.take() {
    //                 self.server_stdin = Some(stdin);
    //             }
    //             // 不等待子进程，让它后台运行
    //         }
    //         Err(e) => eprintln!("Failed to start server: {}", e),
    //     }
    // }
    fn start_server(&mut self) {
        if self.server_process_running {
            return;
        }
        let exe_path = std::path::Path::new("../src/eve-server/eve-server.exe");
        if !exe_path.exists() {
            eprintln!("eve-server.exe not found at {:?}", exe_path);
            return;
        }
        // 使用 tokio::process::Command 以支持异步 I/O
        let tx = self.tx_log.clone();
        tokio::spawn(async move {
            let mut cmd = TokioCommand::new(exe_path);
            cmd.arg("--debug")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
            match cmd.spawn() {
                Ok(mut child) => {
                    // 保存 stdin 用于广播（需将 stdin 传回主线程，此处简化，省略）
                    if let Some(stdout) = child.stdout.take() {
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            use tokio::io::{BufReader, AsyncBufReadExt};
                            let reader = BufReader::new(stdout);
                            let mut lines = reader.lines();
                            while let Ok(Some(line)) = lines.next_line().await {
                                let _ = tx_clone.send(line);
                            }
                        });
                    }
                    if let Some(stderr) = child.stderr.take() {
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            use tokio::io::{BufReader, AsyncBufReadExt};
                            let reader = BufReader::new(stderr);
                            let mut lines = reader.lines();
                            while let Ok(Some(line)) = lines.next_line().await {
                                let _ = tx_clone.send(format!("[STDERR] {}", line));
                            }
                        });
                    }
                    // 等待子进程结束（可选）
                    let status = child.wait().await;
                    println!("Server exited with status: {:?}", status);
                }
                Err(e) => eprintln!("Failed to start server: {}", e),
            }
        });
        // 由于异步，无法立即更新状态，将在日志接收时更新
        self.server_process_running = true; // 乐观设置
        // self.server_status.is_online = false;
        self.is_online = false;
    }

    fn stop_server(&mut self) {
        if let Some(pid) = self.server_pid {
            #[cfg(windows)]
            {
                let _ = Command::new("taskkill")
                    .args(&["/PID", &pid.to_string()])
                    .output();
            }
            #[cfg(unix)]
            {
                let _ = Command::new("kill")
                    .args(&["-15", &pid.to_string()])
                    .output();
            }
            self.server_process_running = false;
            self.server_pid = None;
            self.server_stdin = None;
        }
    }

    // ---------- 实时玩家 ----------
    fn refresh_live_players(&mut self) {
        let pool = self.pool.clone();
        let tx = self.tx_live_players.clone();
        tokio::spawn(async move {
            let rows = sqlx::query(
                "SELECT a.accountName, c.characterName
                 FROM account a
                 JOIN chrCharacters c ON a.accountID = c.accountID
                 WHERE a.online = 1"
            )
            .fetch_all(&*pool)
            .await;
            match rows {
                Ok(rows) => {
                    let players = rows.iter().map(|row| {
                        let account_name: String = row.get(0);
                        let character_name: String = row.get(1);
                        LivePlayer {
                            account_name,
                            character_name,
                            online_since: None,
                        }
                    }).collect();
                    let _ = tx.send(players);
                }
                Err(e) => eprintln!("[ERROR] Load live players: {}", e),
            }
        });
    }

    // ---------- 广播消息 ----------
    fn send_broadcast(&mut self) {
        if self.broadcast_message.is_empty() {
            return;
        }
        if let Some(stdin) = &mut self.server_stdin {
            let cmd = format!("broadcast {}\n", self.broadcast_message);
            if let Err(e) = stdin.write_all(cmd.as_bytes()) {
                self.broadcast_message = format!("Send failed: {}", e);
            } else {
                self.broadcast_message.clear();
            }
        } else {
            self.broadcast_message = "Server not started or stdin not available.".to_string();
        }
    }

    // ---------- 配置编辑 ----------
    fn load_config(&mut self) {
        let path = std::path::Path::new(&self.config_path);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                self.config_content = content;
                self.config_modified = false;
            }
        } else {
            self.config_content = format!("Config file not found at {}", self.config_path);
        }
    }

    fn save_config(&mut self) {
        let path = std::path::Path::new(&self.config_path);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::write(path, &self.config_content) {
            self.config_content = format!("Save failed: {}", e);
        } else {
            self.config_modified = false;
        }
    }

    // ---------- 备份管理 ----------
    fn perform_backup(&mut self) {
    let backup_dir = std::path::Path::new(&self.backup_path);
    if !backup_dir.exists() {
        let _ = std::fs::create_dir_all(backup_dir);
    }
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_file = backup_dir.join(format!("evemu_backup_{}.sql", timestamp));
    let cmd = format!(
        "mysqldump -u evemu -pevemu_password evemu > {}",
        backup_file.display()
    );
    self.backup_status = format!("Backup started: {}", backup_file.display());
    let backup_file_clone = backup_file.clone();
    tokio::spawn(async move {
        #[cfg(windows)]
        let output = TokioCommand::new("cmd")
            .args(&["/C", &cmd])
            .output()
            .await;
        #[cfg(unix)]
        let output = TokioCommand::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .await;
        match output {
            Ok(out) => {
                if out.status.success() {
                    println!("Backup completed: {}", backup_file_clone.display());
                } else {
                    eprintln!("Backup failed: {}", String::from_utf8_lossy(&out.stderr));
                }
            }
            Err(e) => eprintln!("Backup execution error: {}", e),
        }
    });
    self.backup_status = "Backup started in background.".to_string();
    }

    // ---------- 未实现模块 ----------

    fn load_fleets(&mut self) {
        // 模拟数据，实际可从 fleet 表读取
        self.fleets = vec![
            Fleet { id: 1, name: "Alpha Fleet".to_string(), members: 12, system: "Jita".to_string() },
            Fleet { id: 2, name: "Bravo Squad".to_string(), members: 5, system: "Amarr".to_string() },
        ];
    }

    fn load_combat_logs(&mut self) {
        self.combat_logs = vec![
            CombatLog { id: 1, time: "2026-09-08 10:00".to_string(), ship: "Rifter".to_string(), damage: 150, target: "NPC Pirate".to_string() },
        ];
    }

    fn load_station_services(&mut self) {
        self.station_services = vec![
            StationService { id: 1, name: "Repair".to_string(), station: "Jita IV - Moon 4".to_string(), status: "Available".to_string() },
        ];
    }

    fn load_missions(&mut self) {
        self.missions = vec![
            Mission { id: 1, name: "Kill 10 Rats".to_string(), agent: "Agent Smith".to_string(), status: "In Progress".to_string() },
        ];
    }

    fn load_pos_list(&mut self) {
        self.pos_list = vec![
            POS { id: 1, name: "Mining POS".to_string(), system: "Eve".to_string(), state: "Online".to_string() },
        ];
    }

    fn load_corporations(&mut self) {
        self.corporations = vec![
            Corporation { id: 1, name: "EVE University".to_string(), ceo: "Dean".to_string(), members: 120 },
        ];
    }
}

// ---------- 工具函数 ----------
fn parse_ip_port(s: &str) -> Option<(String, u16)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() == 2 {
        let ip = parts[0].trim_matches('[').trim_matches(']').to_string();
        if let Ok(port) = parts[1].parse::<u16>() {
            return Some((ip, port));
        }
    }
    None
}

// ---------- eframe App 实现 ----------
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // 处理通道消息
        while let Ok(accounts) = self.rx_accounts.try_recv() {
            self.accounts = accounts;
            ctx.request_repaint();
        }
        while let Ok(chars) = self.rx_chars.try_recv() {
            self.characters = chars;
            ctx.request_repaint();
        }
        while let Ok(items) = self.rx_items.try_recv() {
            self.items = items;
            ctx.request_repaint();
        }
        while let Ok(status) = self.rx_status.try_recv() {
            self.server_status = status;
            ctx.request_repaint();
        }
        while let Ok(log) = self.rx_log.try_recv() {
            self.log_content = log;
            ctx.request_repaint();
        }
        while let Ok(sql) = self.rx_sql.try_recv() {
            self.sql_output = sql;
            ctx.request_repaint();
        }
        while let Ok(conns) = self.rx_connections.try_recv() {
            self.connections = conns;
            ctx.request_repaint();
        }
        while let Ok(players) = self.rx_live_players.try_recv() {
            self.live_players = players;
            ctx.request_repaint();
        }

        // 侧边栏
        SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("EVEmu Tools");
            ui.separator();
            if ui.selectable_label(self.current_page == Page::Dashboard, "Dashboard").clicked() {
                self.current_page = Page::Dashboard;
            }
            if ui.selectable_label(self.current_page == Page::Accounts, "Accounts").clicked() {
                self.current_page = Page::Accounts;
                self.load_accounts();
            }
            if ui.selectable_label(self.current_page == Page::Characters, "Characters").clicked() {
                self.current_page = Page::Characters;
                if let Some(id) = self.selected_account_for_char {
                    self.load_characters(id);
                }
            }
            if ui.selectable_label(self.current_page == Page::Items, "Items").clicked() {
                self.current_page = Page::Items;
            }
            if ui.selectable_label(self.current_page == Page::SqlConsole, "SQL Console").clicked() {
                self.current_page = Page::SqlConsole;
            }
            if ui.selectable_label(self.current_page == Page::ServerStatus, "Server Status").clicked() {
                self.current_page = Page::ServerStatus;
                self.load_server_status();
            }
            if ui.selectable_label(self.current_page == Page::LogViewer, "Log Viewer").clicked() {
                self.current_page = Page::LogViewer;
                self.load_log();
            }
            if ui.selectable_label(self.current_page == Page::NetworkMonitor, "Network Monitor").clicked() {
                self.current_page = Page::NetworkMonitor;
                self.refresh_connections();
            }
            if ui.selectable_label(self.current_page == Page::ProcessControl, "Process Control").clicked() {
                self.current_page = Page::ProcessControl;
                self.check_server_status();
            }
            if ui.selectable_label(self.current_page == Page::LivePlayers, "Live Players").clicked() {
                self.current_page = Page::LivePlayers;
                self.refresh_live_players();
            }
            if ui.selectable_label(self.current_page == Page::Broadcast, "Broadcast").clicked() {
                self.current_page = Page::Broadcast;
            }
            if ui.selectable_label(self.current_page == Page::ConfigEditor, "Config Editor").clicked() {
                self.current_page = Page::ConfigEditor;
                self.load_config();
            }
            if ui.selectable_label(self.current_page == Page::BackupManager, "Backup Manager").clicked() {
                self.current_page = Page::BackupManager;
            }
            if ui.selectable_label(self.current_page == Page::Help, "Help").clicked() {
                self.current_page = Page::Help;
                self.load_help_commands();
            }
            if ui.selectable_label(self.current_page == Page::Fleets, "Fleets").clicked() {
                self.current_page = Page::Fleets;
                self.load_fleets();
            }
            if ui.selectable_label(self.current_page == Page::Combat, "Combat").clicked() {
                self.current_page = Page::Combat;
                self.load_combat_logs();
            }
            if ui.selectable_label(self.current_page == Page::StationServices, "StationServices").clicked() {
                self.current_page = Page::StationServices;
                self.load_station_services();
            }
            if ui.selectable_label(self.current_page == Page::Missions, "Missions").clicked() {
                self.current_page = Page::Missions;
                self.load_missions();
            }
            if ui.selectable_label(self.current_page == Page::POS, "POS").clicked() {
                self.current_page = Page::POS;
                self.load_pos_list();
            }
            if ui.selectable_label(self.current_page == Page::Corporations, "Corporations").clicked() {
                self.current_page = Page::Corporations;
                self.load_corporations();
            }
            // 类似添加 Combat, StationServices, Missions, POS, Corporations
        });

        // 中央面板
        CentralPanel::default().show(ctx, |ui| {
            match self.current_page {
                Page::Dashboard => self.render_dashboard(ui),
                Page::Accounts => self.render_accounts(ui, ctx),
                Page::Characters => self.render_characters(ui),
                Page::Items => self.render_items(ui),
                Page::SqlConsole => self.render_sql_console(ui),
                Page::ServerStatus => self.render_server_status(ui),
                Page::LogViewer => self.render_log_viewer(ui, ctx),
                Page::NetworkMonitor => self.render_network_monitor(ui),
                Page::ProcessControl => self.render_process_control(ui),
                Page::LivePlayers => self.render_live_players(ui),
                Page::Broadcast => self.render_broadcast(ui),
                Page::ConfigEditor => self.render_config_editor(ui),
                Page::BackupManager => self.render_backup_manager(ui),
                Page::Help => self.render_help(ui),
                // extend mods
                Page::Fleets => self.render_fleets(ui),
                Page::Combat => self.render_combat(ui),
                Page::StationServices => self.render_station_services(ui),
                Page::Missions => self.render_missions(ui),
                Page::POS => self.render_pos(ui),
                Page::Corporations => self.render_corporations(ui),
            }
        });
    }
}

// ---------- 页面渲染 ----------
impl MyApp {
    fn render_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("Dashboard");
        ui.label("Welcome to EVEmu Control Panel");
        ui.separator();
        ui.label(format!("Accounts: {}", self.accounts.len()));
        ui.label(format!("Characters: {}", self.characters.len()));
        ui.label(format!("Items loaded: {}", self.items.len()));
        ui.label(format!("Connections: {}", self.connections.len()));
        ui.label(format!("Server running: {}", self.server_process_running));
        ui.label(format!("Live players: {}", self.live_players.len()));
        // 
        ui.label(format!("Server Version: {}", self.server_version));
        ui.label(format!("Client Build: {}", self.client_build));
        ui.label(format!("Static Data Load: {:.0} ms", self.static_load_ms));
        ui.label(format!("DB Connected: {}", self.db_connected));
        ui.label(format!("Server Online: {}", self.is_online));

    }

    fn render_accounts(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        ui.heading("Account Management");
        if ui.button("Refresh").clicked() {
            self.load_accounts();
        }
        let accounts = self.accounts.clone();
        ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
            for acc in &accounts {
                ui.horizontal(|ui| {
                    ui.label(format!("ID: {}  Name: {}", acc.id, acc.name));
                    if ui.button("Edit").clicked() {
                        self.editing_account = Some(AccountEdit {
                            id: acc.id,
                            name: acc.name.clone(),
                            password: acc.password.clone(),
                            role: acc.role.to_string(),
                        });
                    }
                    if ui.button("Delete").clicked() {
                        self.delete_account(acc.id);
                    }
                });
            }
        });
        ui.separator();
        ui.heading("Add New Account");
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.new_account_name);
        });
        ui.horizontal(|ui| {
            ui.label("Password:");
            ui.text_edit_singleline(&mut self.new_account_pass);
        });
        ui.horizontal(|ui| {
            ui.label("Role:");
            ui.text_edit_singleline(&mut self.new_account_role);
        });
        if ui.add_enabled(!self.new_account_name.is_empty() && !self.new_account_pass.is_empty(), Button::new("Add")).clicked() {
            self.add_account();
        }
        if let Some(edit) = self.editing_account.clone() {
            let mut name = edit.name;
            let mut password = edit.password;
            let mut role = edit.role;
            let id = edit.id;
            Window::new("Edit Account").show(ctx, |ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut name);
                ui.label("Password:");
                ui.text_edit_singleline(&mut password);
                ui.label("Role:");
                ui.text_edit_singleline(&mut role);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.editing_account = None;
                        self.update_account(AccountEdit { id, name, password, role });
                    }
                    if ui.button("Cancel").clicked() {
                        self.editing_account = None;
                    }
                });
            });
        }
    }

    fn render_characters(&mut self, ui: &mut egui::Ui) {
        ui.heading("Character Management");
        ui.horizontal(|ui| {
            ui.label("Select Account:");
            egui::ComboBox::from_label("")
                .selected_text(
                    self.selected_account_for_char
                        .and_then(|id| self.accounts.iter().find(|a| a.id == id).map(|a| a.name.clone()))
                        .unwrap_or_else(|| "None".to_string())
                )
                .show_ui(ui, |ui| {
                    let accounts = self.accounts.clone();
                    for acc in &accounts {
                        if ui.selectable_label(false, &acc.name).clicked() {
                            self.selected_account_for_char = Some(acc.id);
                            self.load_characters(acc.id);
                        }
                    }
                });
        });
        if let Some(acc_id) = self.selected_account_for_char {
            ui.label(format!("Characters for account {}", acc_id));
            let chars = self.characters.clone();
            for chr in &chars {
                ui.horizontal(|ui| {
                    ui.label(&chr.name);
                    if ui.button("Delete").clicked() {
                        let id = chr.id;
                        let acc = acc_id;
                        self.delete_character(id, acc);
                    }
                });
            }
            ui.horizontal(|ui| {
                ui.label("New Character Name:");
                ui.text_edit_singleline(&mut self.new_char_name);
                if ui.add_enabled(!self.new_char_name.is_empty(), Button::new("Add")).clicked() {
                    self.add_character(acc_id);
                }
            });
        } else {
            ui.label("Select an account to view characters.");
        }
    }

    fn render_items(&mut self, ui: &mut egui::Ui) {
        ui.heading("Item Management");
        ui.horizontal(|ui| {
            ui.label("Filter by owner (account/character ID):");
            ui.text_edit_singleline(&mut self.filter_owner);
            if ui.button("Search").clicked() {
                let owner = self.filter_owner.parse::<u32>().ok();
                self.load_items(owner);
            }
        });
        if !self.items.is_empty() {
            ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                for item in &self.items {
                    ui.label(
                        format!("ID: {}  Type: {}  Owner: {}  Qty: {}  Loc: {}",
                            item.id, item.type_id, item.owner_id, item.quantity, item.location_id)
                    );
                }
            });
        } else {
            ui.label("No items loaded. Search to display.");
        }
    }

    fn render_sql_console(&mut self, ui: &mut egui::Ui) {
        ui.heading("SQL Console");
        ui.label("Execute SQL statements:");
        ui.text_edit_multiline(&mut self.sql_input);
        if ui.button("Execute").clicked() {
            self.execute_sql();
        }
        ui.label("Output:");
        ui.text_edit_multiline(&mut self.sql_output);
    }

    fn render_server_status(&mut self, ui: &mut egui::Ui) {
        ui.heading("Server Status");
        if ui.button("Refresh Status").clicked() {
            self.load_server_status();
        }
        ui.label(format!("Online Players: {}", self.server_status.online_players));
        ui.label(format!("Uptime: {} seconds", self.server_status.uptime_seconds));
        if let Some(t) = self.server_status.last_update {
            ui.label(format!("Last Update: {:?}", t));
        } else {
            ui.label("Last Update: Never");
        }
    }

    // fn parse_server_info(&mut self, line: &str) {
    //     if let Some(cap) = regex::Regex::new(r"Server Revision:\s+(\S+)").unwrap().captures(line) {
    //         self.server_version = cap[1].to_string();
    //     }
    //     if let Some(cap) = regex::Regex::new(r"Client Build:\s+(\d+)").unwrap().captures(line) {
    //         self.client_build = cap[1].parse().unwrap_or(0);
    //     }
    //     if let Some(cap) = regex::Regex::new(r"Static Data loaded in (\d+\.?\d*)ms").unwrap().captures(line) {
    //         self.static_load_ms = cap[1].parse().unwrap_or(0.0);
    //     }
    //     if line.contains("DataBase Connected") {
    //         self.db_connected = true;
    //     }
    //     if line.contains("TCP Server started on port") {
    //         self.tcp_started = true;
    //     }
    //     if line.contains("EVEmu Server is Online") {
    //         self.is_online = true;
    //     }
    // }
    fn parse_server_info(&mut self, line: &str) {
        if line.contains("Server Revision:") {
            if let Some(parts) = line.split("Server Revision:").nth(1) {
                self.server_version = parts.trim().to_string();
            }
        }
        if line.contains("Client Build:") {
            if let Some(parts) = line.split("Client Build:").nth(1) {
                if let Ok(build) = parts.trim().parse::<u32>() {
                    self.client_build = build;
                }
            }
        }
        if line.contains("Static Data loaded in") {
            if let Some(parts) = line.split("Static Data loaded in").nth(1) {
                if let Some(ms) = parts.split("ms").next() {
                    if let Ok(val) = ms.trim().parse::<f64>() {
                        self.static_load_ms = val;
                    }
                }
            }
        }
        if line.contains("DataBase Connected") {
            self.db_connected = true;
        }
        if line.contains("TCP Server started on port") {
            self.tcp_started = true;
        }
        if line.contains("EVEmu Server is Online") {
            self.is_online = true;
        }
    }

    // 添加两个增强功能：
// 颜色日志显示：在 Log Viewer 中根据日志级别（E、W、G、B、Y、C）着色。
// 帮助命令面板：新增 Help 页面，显示 EVEmu 控制台所有可用命令及其描述。
// 前缀说明：E=错误，W=警告，G=信息，B=调试，Y=特殊，C=连接/状态。


    // fn render_log_viewer(&mut self, ui: &mut egui::Ui, ctx: &Context) {
    //     ui.heading("Log Viewer");
    //     ui.checkbox(&mut self.log_auto_refresh, "Auto-refresh");
    //     if ui.button("Refresh Log").clicked() {
    //         self.load_log();
    //     }
    //     ScrollArea::vertical().show(ui, |ui| {
    //         ui.label(&self.log_content);
    //     });
    //     if self.log_auto_refresh {
    //         ctx.request_repaint_after(std::time::Duration::from_secs(3));
    //     }
    // }
    // fn render_log_viewer(&mut self, ui: &mut egui::Ui, ctx: &Context) {
    //     ui.heading("Log Viewer");
    //     // src/logs/07-09-2026-15-10.log
    //     ui.label(format!("Log lines: {}", self.log_content.lines().count()));
    //     ui.checkbox(&mut self.log_auto_refresh, "Auto-refresh");
    //     if ui.button("Refresh Log").clicked() {
    //         self.load_log();
    //     }
    //     ScrollArea::vertical().show(ui, |ui| {
    //         for line in self.log_content.lines() {
    //             let color = if line.starts_with("E ") {
    //                 egui::Color32::RED
    //             } else if line.starts_with("W ") {
    //                 egui::Color32::YELLOW
    //             } else if line.starts_with("G ") {
    //                 egui::Color32::GREEN
    //             } else if line.starts_with("B ") {
    //                 egui::Color32::LIGHT_BLUE
    //             } else if line.starts_with("Y ") {
    //                 egui::Color32::GOLD
    //             } else if line.starts_with("C ") {
    //                 egui::Color32::LIGHT_BLUE
    //             } else {
    //                 egui::Color32::WHITE
    //             };
    //             ui.colored_label(color, line);
    //         }
    //     });
    //     if self.log_auto_refresh {
    //         ctx.request_repaint_after(std::time::Duration::from_secs(2));
    //     }
    // }
    
    fn render_log_viewer(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        ui.heading("Log Viewer");
        ui.checkbox(&mut self.log_auto_refresh, "Auto-refresh");
        if ui.button("Refresh Log").clicked() {
            self.load_log();
        }

        // 显示当前日志行数（调试信息）
        ui.label(format!("Lines: {}", self.log_content.lines().count()));

        // 如果日志为空，显示提示（在滚动区域外部和内部各放一份）
        if self.log_content.is_empty() {
            ui.label("No logs loaded. Click 'Refresh Log' to load.");
        }

        // 滚动区域：设置最小滚动高度，确保可见
        ScrollArea::vertical()
            .min_scrolled_height(300.0)   // 关键修复：使用 min_scrolled_height
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                if self.log_content.is_empty() {
                    ui.label("(No log content)");
                } else {
                    for line in self.log_content.lines() {
                        let color = if line.starts_with("E ") {
                            egui::Color32::RED
                        } else if line.starts_with("W ") {
                            egui::Color32::YELLOW
                        } else if line.starts_with("G ") {
                            egui::Color32::GREEN
                        } else if line.starts_with("B ") {
                            egui::Color32::LIGHT_BLUE
                        } else if line.starts_with("Y ") {
                            egui::Color32::GOLD
                        } else if line.starts_with("C ") {
                            egui::Color32::LIGHT_BLUE
                        } else {
                            egui::Color32::WHITE
                        };
                        ui.colored_label(color, line);
                    }
                }
            });

        if self.log_auto_refresh {
            ctx.request_repaint_after(std::time::Duration::from_secs(2));
        }
    }


    fn load_help_commands(&mut self) {
        self.help_commands = vec![
            ("h".to_string(), "Displays this dialog.".to_string()),
            ("e".to_string(), "Exits the server, saving all loaded items.".to_string()),
            ("c".to_string(), "Displays connected clients.".to_string()),
            ("s".to_string(), "Displays Server's System Status.".to_string()),
            ("v".to_string(), "Displays server version.".to_string()),
            ("i".to_string(), "Displays server information.".to_string()),
            ("a".to_string(), "Immediately saves all loaded items. *Broken*".to_string()),
            ("b".to_string(), "Broadcasts a message to all clients. *Not Implemented*".to_string()),
            ("n".to_string(), "Broadcasts a message via notification window.".to_string()),
            ("m".to_string(), "Broadcasts a message via message window.".to_string()),
            ("p".to_string(), "Prints a profile of current server runtimes. *Incomplete*".to_string()),
            ("o".to_string(), "Prints a list of common roles and their values.".to_string()),
            ("c".to_string(), "Prints a list of currently loaded Commands. (long list)".to_string()),
            ("t".to_string(), "Prints the current test object.".to_string()),
            ("f".to_string(), "Compiles and prints all item effects.".to_string()),
            ("d".to_string(), "Prints a list of current threads.".to_string()),
            ("l".to_string(), "Reloads log.ini to change values without restarting.".to_string()),
            ("q".to_string(), "Prints current statistic data.".to_string()),
            ("r".to_string(), "Echo all chat msgs to console. *Not Implemented*".to_string()),
            ("h".to_string(), "Displays this dialog.".to_string()), // 重复，但保持
        ];
    }
    
    // help (./eve-server command line)
    fn render_help(&mut self, ui: &mut egui::Ui) {
        ui.heading("EVEmu Console Commands");
        ui.label("Available commands (from 'h' command):");
        ui.separator();
        ScrollArea::vertical().show(ui, |ui| {
            for (cmd, desc) in &self.help_commands {
                ui.horizontal(|ui| {
                    ui.label(format!("({})", cmd));
                    ui.label(desc);
                });
            }
        });
    }

    fn render_network_monitor(&mut self, ui: &mut egui::Ui) {
        ui.heading("Network Monitor");
        ui.label("Active TCP connections on ports 26000/26001 (from netstat)");
        if ui.button("Refresh").clicked() {
            self.refresh_connections();
        }
        if self.connections.is_empty() {
            ui.label("No connections found.");
        } else {
            ScrollArea::vertical().max_height(500.0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Src IP"); ui.separator();
                    ui.label("Src Port"); ui.separator();
                    ui.label("Dst IP"); ui.separator();
                    ui.label("Dst Port"); ui.separator();
                    ui.label("State");
                });
                for conn in &self.connections {
                    ui.horizontal(|ui| {
                        ui.label(&conn.src_ip);
                        ui.label(conn.src_port.to_string());
                        ui.label(&conn.dst_ip);
                        ui.label(conn.dst_port.to_string());
                        ui.label(&conn.state);
                    });
                }
            });
        }
    }

    fn render_process_control(&mut self, ui: &mut egui::Ui) {
        ui.heading("Server Process Control");
        ui.label(format!("Running: {}", self.server_process_running));
        if let Some(pid) = self.server_pid {
            ui.label(format!("PID: {}", pid));
        }
        ui.horizontal(|ui| {
            if ui.button("Start Server").clicked() {
                self.start_server();
            }
            if ui.button("Stop Server").clicked() {
                self.stop_server();
            }
            if ui.button("Refresh Status").clicked() {
                self.check_server_status();
            }
        });
        ui.label("Note: Assumes eve-server.exe is at ../src/eve-server/eve-server.exe");
    }

    // ---------- 新增：Live Players ----------
    fn render_live_players(&mut self, ui: &mut egui::Ui) {
        ui.heading("Live Players");
        if ui.button("Refresh").clicked() {
            self.refresh_live_players();
        }
        if self.live_players.is_empty() {
            ui.label("No players online.");
        } else {
            ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Account"); ui.separator();
                    ui.label("Character"); ui.separator();
                    ui.label("Online Since");
                });
                for player in &self.live_players {
                    ui.horizontal(|ui| {
                        ui.label(&player.account_name);
                        ui.label(&player.character_name);
                        ui.label("Unknown"); // 简化
                    });
                }
            });
        }
    }

    // ---------- 新增：Broadcast ----------
    fn render_broadcast(&mut self, ui: &mut egui::Ui) {
        ui.heading("Broadcast Message");
        ui.label("Send a message to all online players.");
        ui.text_edit_multiline(&mut self.broadcast_message);
        if ui.button("Send Broadcast").clicked() {
            self.send_broadcast();
        }
        ui.label("Note: Server must be running and stdin available.");
    }

    // ---------- 新增：Config Editor ----------
    fn render_config_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("Server Configuration Editor");
        ui.label(format!("Editing: {}", self.config_path));
        if ui.button("Reload from File").clicked() {
            self.load_config();
        }
        if ui.button("Save to File").clicked() {
            self.save_config();
        }
        ui.label(if self.config_modified { "Unsaved changes" } else { "Saved" });
        ScrollArea::vertical().max_height(500.0).show(ui, |ui| {
            ui.text_edit_multiline(&mut self.config_content);
        });
    }

    // ---------- 新增：Backup Manager ----------
    fn render_backup_manager(&mut self, ui: &mut egui::Ui) {
        ui.heading("Database Backup");
        ui.label(format!("Backup directory: {}", self.backup_path));
        if ui.button("Perform Backup Now").clicked() {
            self.perform_backup();
        }
        ui.label(&self.backup_status);
        ui.label("Note: Requires mysqldump in PATH and correct credentials in config.");
    }



    // ---------- 新增：Extended Mods ----------

    fn render_fleets(&mut self, ui: &mut egui::Ui) {
        ui.heading("🚧 Fleets System (Under Development)");
        ui.label("Fleets allow grouping of players for coordinated actions.");
        if ui.button("Refresh").clicked() {
            self.load_fleets();
        }
        if self.fleets.is_empty() {
            ui.label("No fleet data available.");
        } else {
            ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("ID"); ui.separator();
                    ui.label("Name"); ui.separator();
                    ui.label("Members"); ui.separator();
                    ui.label("System");
                });
                for fleet in &self.fleets {
                    ui.horizontal(|ui| {
                        ui.label(fleet.id.to_string());
                        ui.label(&fleet.name);
                        ui.label(fleet.members.to_string());
                        ui.label(&fleet.system);
                    });
                }
            });
        }
    }

    fn render_combat(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚔️ Combat System (Early Stage)");
        ui.label("Module activation, damage calculation, and ship combat.");
        if ui.button("Refresh").clicked() {
            self.load_combat_logs();
        }
        if self.combat_logs.is_empty() {
            ui.label("No combat logs.");
        } else {
            ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Time"); ui.separator();
                    ui.label("Ship"); ui.separator();
                    ui.label("Damage"); ui.separator();
                    ui.label("Target");
                });
                for log in &self.combat_logs {
                    ui.horizontal(|ui| {
                        ui.label(&log.time);
                        ui.label(&log.ship);
                        ui.label(log.damage.to_string());
                        ui.label(&log.target);
                    });
                }
            });
        }
    }

    fn render_station_services(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏢 Station Services (Partial)");
        ui.label("Repair, cloning, insurance, and other station services.");
        if ui.button("Refresh").clicked() {
            self.load_station_services();
        }
        if self.station_services.is_empty() {
            ui.label("No services available.");
        } else {
            ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name"); ui.separator();
                    ui.label("Station"); ui.separator();
                    ui.label("Status");
                });
                for svc in &self.station_services {
                    ui.horizontal(|ui| {
                        ui.label(&svc.name);
                        ui.label(&svc.station);
                        ui.label(&svc.status);
                    });
                }
            });
        }
    }

    fn render_missions(&mut self, ui: &mut egui::Ui) {
        ui.heading("📋 Missions & NPCs (Not Implemented)");
        ui.label("Mission offers, agent conversations, and rewards.");
        if ui.button("Refresh").clicked() {
            self.load_missions();
        }
        if self.missions.is_empty() {
            ui.label("No missions available.");
        } else {
            ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name"); ui.separator();
                    ui.label("Agent"); ui.separator();
                    ui.label("Status");
                });
                for mission in &self.missions {
                    ui.horizontal(|ui| {
                        ui.label(&mission.name);
                        ui.label(&mission.agent);
                        ui.label(&mission.status);
                    });
                }
            });
        }
    }

    fn render_pos(&mut self, ui: &mut egui::Ui) {
        ui.heading("🛰️ Player Owned Stations (Not Implemented)");
        ui.label("POS structures, modules, and defenses.");
        if ui.button("Refresh").clicked() {
            self.load_pos_list();
        }
        if self.pos_list.is_empty() {
            ui.label("No POS found.");
        } else {
            ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name"); ui.separator();
                    ui.label("System"); ui.separator();
                    ui.label("State");
                });
                for pos in &self.pos_list {
                    ui.horizontal(|ui| {
                        ui.label(&pos.name);
                        ui.label(&pos.system);
                        ui.label(&pos.state);
                    });
                }
            });
        }
    }

    fn render_corporations(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏛️ Corporations & Alliances (Partial)");
        ui.label("Corporation management, roles, and alliances.");
        if ui.button("Refresh").clicked() {
            self.load_corporations();
        }
        if self.corporations.is_empty() {
            ui.label("No corporation data.");
        } else {
            ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name"); ui.separator();
                    ui.label("CEO"); ui.separator();
                    ui.label("Members");
                });
                for corp in &self.corporations {
                    ui.horizontal(|ui| {
                        ui.label(&corp.name);
                        ui.label(&corp.ceo);
                        ui.label(corp.members.to_string());
                    });
                }
            });
        }
    }




}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = Arc::new(MySqlPool::connect(&db_url).await?);

    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport = egui::ViewportBuilder::default()
        .with_inner_size(egui::vec2(1000.0, 750.0));

    eframe::run_native(
        "EVEmu Control Panel",
        native_options,
        Box::new(|_cc| Box::new(MyApp::new(pool))),
    )
    .unwrap();
    Ok(())
}