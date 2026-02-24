# 气藏工程计算（Nature Gas Engineering Calculator）

## 项目简介

该项目是一个使用 Rust 编写的轻量级气藏/气井工程计算与管理后台，提供常用的 PVT、密度、Z 因子、比重等计算接口，并带有前端页面用于交互式批量计算与结果展示。

后端使用 `axum` 提供 HTTP API，使用 `sqlx` 连接 MySQL 存储部分数据；前端为静态 HTML/JS/CSS 文件，项目包含批量计算所需的表格交互（handsontable）。

## 主要特性

- 单井或批量 PVT / PB / 密度 / 组分比重 计算接口
- 登录认证（简单示例，可扩展）
- 静态前端页面：计算界面、登录、首页
- 使用全局连接池以支持并发请求

## 仓库结构（重点）

- `src/`：Rust 源代码
  - `main.rs`：程序入口、路由与静态文件挂载（监听端口：3000）
  - `handlers.rs`：HTTP 请求处理器（API 实现）
  - `models.rs`：数据模型与序列化定义
  - `pressure.rs`：与压力/物性相关的计算逻辑
- `html/`：前端 HTML 页面（登录/首页/计算页面）
- `assets/`：前端静态资源（CSS、JS、handsontable 等）
- `Cargo.toml`：Rust 依赖与构建配置

（查看核心入口： [src/main.rs](src/main.rs)）

## 运行前提

- 已安装 Rust（建议 stable 或 1.60+），以及 `cargo` 工具链
- 已安装并可访问的 MySQL 数据库（若不使用 DB，可忽略相关接口）
- 环境变量文件 `.env`（放置在项目根目录）应包含：

```
DATABASE_URL=mysql://user:password@host:port/database_name
```

## 本地构建与运行

1. 获取代码并进入项目根目录

```
git clone <repo-url>
cd nature_gas_engineering_calculator
```

2. 设置 `.env`（参见上文）
3. 构建并运行（开发模式）

```
cargo run
```

4. 访问服务

打开浏览器访问：

```
http://localhost:3000
```

默认根路径会重定向到登录页（`/login.html`）。静态资源路径通过 `assets/` 提供。

若需要以发布模式运行：

```
cargo run --release
```

## 可用 HTTP 接口（后端路由说明）

以下为主要 API 路径（均为 POST 接口，详见 `src/handlers.rs`）：

- `POST /api/login` — 登录认证
- `POST /api/getWellData` — 获取井/测点数据
- `POST /api/calculateZ` — 计算 Z 因子
- `POST /api/calculateBg` — 计算气体比体积（Bg）
- `POST /api/calculateCg` — 计算气体体积分数（Cg）
- `POST /api/calculateDensity` — 计算密度
- `POST /api/calculateNiandu` — 计算黏度或相关参数（按实现）
- `POST /api/calculatePwbs` — 井筒/井口压力相关计算
- `POST /api/calculateBatchPVT` — 批量 PVT 计算（用于前端表格导入）
- `POST /api/calculateBatchPb` — 批量 PB 计算

（更多实现细节请参见： [src/handlers.rs](src/handlers.rs)）

## 前端页面

- 登录页： `html/login.html`
- 首页： `html/index.html`
- 批量 PVT 页面： `html/calculate_pvt.html`
- 批量 PB 页面： `html/calculate_pb.html`

前端交互静态资源位于 `assets/`，其中包含 `handsontable` 的相关 JS/CSS 用于表格编辑与批量计算上传。

## 开发提示

- 在开发过程中可设置环境变量 `RUST_LOG=debug` 以获得更多日志输出。
- 若要调试数据库连接，先确保 `DATABASE_URL` 正确并能从当前主机访问 MySQL。
- 路由与功能扩展在 `src/handlers.rs` 中添加新路由并实现处理器。

## 测试与示例

目前仓库未包含自动化测试用例。后续拟为核心计算函数（如 `pressure.rs` 中的方法）补充单元测试，放在相应 `#[cfg(test)]` 模块下。

## 贡献指南

欢迎 PR 与 Issue。提交代码前请保持代码风格一致并提供简洁的变更说明。若涉及数据库方案变更，请在 PR 描述中说明迁移步骤。

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

---

## 部署与服务器配置

### 生产环境部署

1. 构建发布版本：

   ```
   cargo build --release
   ```

2. 设置环境变量（生产数据库URL等）。
3. 运行：

   ```
   ./target/release/nature_gas_engineering_calculator
   ```

### 服务器配置建议（支持500并发连接）

- **CPU**：4-8核（推荐多核处理器，支持异步处理）。
- **内存**：8-16GB RAM。
- **存储**：SSD至少100GB。
- **网络**：1Gbps带宽。
- **数据库**：MySQL单独部署，配置max_connections=500+，InnoDB缓冲池8-16GB。
- **操作系统**：Linux（如Ubuntu 22.04）。

建议使用云服务器如AWS EC2或阿里云ECS，并进行负载测试验证。
