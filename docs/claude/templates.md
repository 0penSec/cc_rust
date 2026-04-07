# 代码模板

> 常用的代码结构和模板示例

---

## 函数文档模板

### 完整版（公共 API）

```rust
/// 功能简述（一句话说明）
///
/// ## 参数
///
/// - `param1`: 参数说明
/// - `param2`: 参数说明
///
/// ## 返回值
///
/// 返回值说明
///
/// ## 为什么这样设计
///
/// 解释设计决策：
/// - 为什么选择这个方案
/// - 有什么权衡取舍
/// - 参考了什么设计模式
///
/// ## 使用示例
///
/// ```rust
/// use crate_name::function_name;
///
/// # async fn example() -> Result<()> {
/// let result = function_name(arg1, arg2).await?;
/// assert_eq!(result, expected);
/// # Ok(())
/// # }
/// ```
///
/// ## 注意事项
///
/// - 边界情况说明
/// - 性能考虑
/// - 线程安全等
///
/// ## 错误处理
///
/// 可能返回的错误：
/// - `ErrorType1`: 说明
/// - `ErrorType2`: 说明
pub async fn function_name(param1: Type1, param2: Type2) -> Result<ReturnType> {
    // 实现
}
```

### 简化版（内部函数）

```rust
/// 功能简述
///
/// ## 为什么需要
///
/// 简要说明设计原因
fn internal_function(param: Type) -> ReturnType {
    // 实现
}
```

---

## 结构体模板

```rust
/// 结构体名称
///
/// ## 为什么需要这个结构
///
/// 说明结构的职责和设计目的
///
/// ## 使用场景
///
/// - 场景1
/// - 场景2
///
/// ## 线程安全
///
/// 说明是否线程安全，为什么
#[derive(Debug, Clone)]
pub struct StructName {
    /// 字段1说明
    pub field1: Type1,
    
    /// 字段2说明
    /// 
    /// 如果字段复杂，可以详细说明
    field2: Type2,
}

impl StructName {
    /// 创建新实例
    ///
    /// ## 参数
    ///
    /// - `param`: 参数说明
    pub fn new(param: Type) -> Self {
        Self {
            field1: default_value,
            field2: param,
        }
    }
}
```

---

## 枚举模板

```rust
/// 枚举名称
///
/// ## 为什么需要
///
/// 说明枚举的用途
///
/// ## 变体说明
///
/// - `Variant1`: 说明
/// - `Variant2`: 说明
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumName {
    /// 变体1说明
    Variant1,
    
    /// 变体2说明
    /// 
    /// 如果带数据，说明数据含义
    Variant2(String),
    
    /// 变体3说明
    Variant3 { field: Type },
}

impl EnumName {
    /// 获取字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Variant1 => "variant1",
            Self::Variant2(_) => "variant2",
            Self::Variant3 { .. } => "variant3",
        }
    }
}
```

---

## 模块文档模板

```rust
//! 模块名称
//!
//! 模块功能概述（2-3句话）
//!
//! ## 核心设计思路
//!
//! 1. **设计点1**: 解释
//! 2. **设计点2**: 解释
//!
//! ## 目录结构
//!
//! ```text
//! module/
//! ├── file1.rs    # 说明
//! └── file2.rs    # 说明
//! ```
//!
//! ## 使用示例
//!
//! ```rust
//! use crate::module::{Type, function};
//!
//! # async fn example() -> Result<()> {
//! let instance = Type::new();
//! let result = function(&instance).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## 模块关系
//!
//! - 依赖 `crate::other_module`
//! - 被 `crate::parent_module` 使用

pub mod sub_module;

pub use sub_module::{Type, function};

// 模块实现...
```

---

## 测试模板

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// 测试功能名称
    ///
    /// ## 验证点
    ///
    /// 1. 验证点1
    /// 2. 验证点2
    ///
    /// ## 为什么重要
    ///
    /// 说明测试的必要性
    #[test]
    fn test_feature() {
        // 准备
        let input = "test data";
        
        // 执行
        let result = function_under_test(input);
        
        // 验证
        assert_eq!(result, expected);
        assert!(result.is_ok());
    }

    /// 测试错误处理
    #[test]
    fn test_feature_error() {
        let result = function_under_test("invalid");
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ErrorType::SpecificError));
    }
}
```

### 异步测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// 测试异步功能
    #[tokio::test]
    async fn test_async_feature() {
        let storage = create_test_storage().await;
        
        let result = storage.async_method().await;
        
        assert!(result.is_ok());
    }

    /// 测试并发场景
    #[tokio::test]
    async fn test_concurrent_access() {
        let storage = Arc::new(create_test_storage().await);
        let mut handles = vec![];
        
        for i in 0..10 {
            let s = Arc::clone(&storage);
            handles.push(tokio::spawn(async move {
                s.operation(i).await.unwrap();
            }));
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
    }
}
```

---

## 错误处理模板

```rust
use thiserror::Error;

/// 模块错误类型
#[derive(Debug, Error)]
pub enum ModuleError {
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// 无效输入
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// 未找到
    #[error("Not found: {0}")]
    NotFound(String),
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, ModuleError>;
```

---

## 快速命令模板

### 提交前检查

```bash
cargo fmt --all \
  && cargo clippy --all-targets --all-features -- -D warnings \
  && cargo test --all \
  && cargo doc --all --no-deps
```

### 快速提交

```bash
git add -A \
  && git commit -m "type: 描述" \
  && git push origin main
```

### 修复后强制更新

```bash
cargo fmt --all \
  && git add -A \
  && git commit --amend --no-edit \
  && git push origin main --force-with-lease
```

---

*参考主文档：[../CLAUDE.md](../CLAUDE.md)*
