# EVEmu Control Panel (Rust)

一个用 Rust + egui + sqlx 重写的 EVEmu 服务器管理工具，用于管理 EVE Online 私服（EVEmu Crucible）的账户。

## ✨ 功能


- **Dashboard**：概览服务器数据（账户数、角色数）
- **账户管理**：
  - 查看账户列表（ID、用户名、角色权限）
  - 添加新账户（用户名、密码、角色）
  - 编辑账户（修改用户名、密码、角色）
  - 删除账户
- **角色管理**（框架，待实现）
- **物品管理**（框架，待实现）
- **SQL 控制台**（框架，待实现）
- **服务器状态监控**（框架，待实现）
- **日志查看**（框架，待实现）
- **网络监控**（新增）：实时显示 TCP 连接（端口 26000/26001），自动更新连接表
- **进程控制**（新增）：启动/停止 `eve-server.exe`，显示运行状态和 PID

- 舰队系统 (Fleets) 管理页面
- 模块与战斗 (Combat) 日志页面
- 空间站服务 (Station Services) 状态页面
- 任务与 NPC (Missions) 页面
- 玩家 owned 空间站 (POS) 列表页面
- 军团与联盟 (Corporations) 信息页面
- 后台任务管理器 (background.rs) 支持定时任务

- **页面	功能	实现方式
Live Players	显示在线玩家列表（账号+角色）	查询 account.online=1 JOIN chrCharacters
Broadcast	向所有在线玩家发送消息	通过服务器进程的 stdin 发送 broadcast 命令
Config Editor	编辑 eve-server.xml	读取/写入文件，支持保存
Backup Manager	备份数据库	调用 mysqldump 导出 SQL

## 🛠️ 技术栈

- **Rust 1.98+**（GNU 工具链）
- **egui 0.27** 即时模式 GUI
- **sqlx 0.8** 异步 MySQL 驱动
- **tokio 1.x** 异步运行时
- **dotenvy 0.15** 环境变量管理
- **pcap 0.9** 数据包捕获（网络监控）
- **sysinfo 0.30** 系统信息（进程控制）
- **chrono 0.4** 时间处理


## 🚀 编译与运行

### 前置条件

- Rust 工具链（x86_64-pc-windows-gnu 或 msvc）
- MariaDB/MySQL 服务器（版本 >= 10.0）
- EVEmu 数据库已初始化（evemu 数据库）

### 设置数据库连接

默认连接字符串为：

mysql://evemu:evemu_password@localhost/evemu?ssl-mode=disabled

text

如需修改，编辑 src/main.rs 中的 db_url 变量。

### 编译运行

`powershell
cd evemu_tool
cargo run
首次编译会下载依赖，请耐心等待。若 C 盘空间不足，可设置目标目录到其他盘：

powershell
 = "D:\rust_target"
cargo run
📋 使用说明
启动 MariaDB 服务（mysqld.exe --console）

运行程序

界面显示账户列表

输入用户名、密码、角色（数值），点击“Add Account”添加

点击“Refresh List”刷新列表

生成发布版本
powershell
cargo build --release
可执行文件位于 target/release/evemu_tool.exe。

界面预览
Dashboard：概览所有关键数据
Accounts：账户管理（CRUD）
Characters：角色管理
Items：物品查询
SQL Console：执行 SQL
Server Status：服务器状态
Log Viewer：实时日志
Network Monitor：连接列表
Process Control：启动/停止服务器
Live Playrs：在线玩家
Broadcast：广播消息
Config Editor：配置编辑
Backup Manager：数据库备份

###前缀说明：E=错误，W=警告，G=信息，B=调试，Y=特殊，C=连接/状态。


⚠️ 注意事项
密码在数据库中明文存储（EVEmu 默认行为），建议生产环境使用哈希加密。

角色权限值请参考 EVEmu 文档，默认 7131450020691447808 为 ROLE_DEV。

编辑和删除操作会立即生效，请谨慎操作。

###🧩 未来扩展计划
☑ 账户管理（CRUD）
☑ 角色管理（按账户筛选、CRUD）
☑ 物品管理（按所有者查询、列表）
☑ SQL 控制台（SELECT/INSERT/UPDATE/DELETE）
☑ 服务器状态（在线玩家数）
☑ 日志查看（手动刷新、自动刷新）
☑ 网络监控（TCP 连接列表）
☑ 进程控制（启动/停止服务器）
□ 实时玩家列表与踢出
□ 广播消息
□ 服务器配置编辑（eve-server.xml）
□ 自动备份数据库

⚠️ 注意事项
密码明文存储（EVEmu 默认），如需加密需自行扩展

角色值请参考 EVEmu 文档（默认 7131450020691447808 为 ROLE_DEV）

确保 MariaDB 端口 3306 未被占用

📄 许可证
沿用 EVEmu 项目的 LGPL 2.1 许可证（由原控制面板继承）。

🔗 相关项目
EVEmu Crucible - 服务器核心
evemu_control_panel (C#) - 原版控制面板
