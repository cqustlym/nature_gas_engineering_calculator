```rust
// #[tokio::main]
// async fn main() -> Result<()> {
//     dotenv().ok();
//     let url   = env::var("DATABASE_URL")?;
//     let pool  = MySqlPool::connect(&url).await?;   // 1. 建池
//
//     //核心代码，根据井号在数据库里读取参数，加上给定的参数后，计算得到结果
//     let data = pressure::query_z(&pool, "天东9").await?;
//     let rlt = z(data.0.unwrap(),data.1.unwrap(),data.2.unwrap(),20);
//     println!("{rlt}");
//     Ok(())
// }
```
