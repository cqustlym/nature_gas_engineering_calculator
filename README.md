# 气藏工程计算

## 主要功能

- 常用的气藏及井筒相关计算功能
- 气藏及气井数据查询功能

## 技术栈

- 后端：Rust
- 前端：HTML+CSS+Javascript
- 数据库：Mysql

## 用到的第三方库

- dotenv
  加载环境变量中的.env文件，确保重要信息（eg.账号、数据库密码登）不泄露
- sqlx
  Rust与Mysql数据库交互
- axum
  Rust的web框架
- serde
  Rust的序列化和反序列化库
- once_cell

## Todo List

- [ ] 编写其他计算函数
