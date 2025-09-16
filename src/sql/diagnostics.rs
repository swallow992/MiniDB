//! Smart error diagnostics for MiniDB
//!
//! This module provides enhanced error reporting with intelligent suggestions:
//! - Keyword spelling suggestions  
//! - Table/column name recommendations
//! - Syntax error hints
//! - Context-aware error messages

use std::collections::HashMap;

/// 诊断建议类型
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionType {
    /// SQL关键字拼写建议
    KeywordSpelling,
    /// 表名建议
    TableName,
    /// 列名建议
    ColumnName,
    /// 语法提示
    SyntaxHint,
    /// 示例用法
    ExampleUsage,
}

/// 错误诊断建议
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// 建议类型
    pub suggestion_type: SuggestionType,
    /// 建议内容
    pub text: String,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
}

impl Suggestion {
    /// 创建新建议
    pub fn new(suggestion_type: SuggestionType, text: String, confidence: f64) -> Self {
        Self {
            suggestion_type,
            text,
            confidence,
        }
    }
}

/// 智能诊断引擎
pub struct DiagnosticEngine {
    /// SQL关键字列表
    sql_keywords: Vec<String>,
    /// 常见错误模式
    common_patterns: HashMap<String, Vec<String>>,
}

impl DiagnosticEngine {
    /// 创建新的诊断引擎
    pub fn new() -> Self {
        let sql_keywords = vec![
            "SELECT".to_string(), "FROM".to_string(), "WHERE".to_string(),
            "INSERT".to_string(), "INTO".to_string(), "VALUES".to_string(),
            "UPDATE".to_string(), "SET".to_string(), "DELETE".to_string(),
            "CREATE".to_string(), "TABLE".to_string(), "DROP".to_string(),
            "ALTER".to_string(), "INDEX".to_string(), "PRIMARY".to_string(),
            "KEY".to_string(), "FOREIGN".to_string(), "REFERENCES".to_string(),
            "NOT".to_string(), "NULL".to_string(), "DEFAULT".to_string(),
            "UNIQUE".to_string(), "CHECK".to_string(), "CONSTRAINT".to_string(),
            "AND".to_string(), "OR".to_string(), "IN".to_string(),
            "LIKE".to_string(), "ORDER".to_string(), "BY".to_string(),
            "GROUP".to_string(), "HAVING".to_string(), "LIMIT".to_string(),
            "OFFSET".to_string(), "INNER".to_string(), "LEFT".to_string(),
            "RIGHT".to_string(), "FULL".to_string(), "JOIN".to_string(),
            "ON".to_string(), "AS".to_string(), "DISTINCT".to_string(),
            "COUNT".to_string(), "SUM".to_string(), "AVG".to_string(),
            "MIN".to_string(), "MAX".to_string(), "INTEGER".to_string(),
            "VARCHAR".to_string(), "TEXT".to_string(), "BOOLEAN".to_string(),
            "DATE".to_string(), "TIME".to_string(), "TIMESTAMP".to_string(),
        ];

        let mut common_patterns = HashMap::new();
        
        // 常见拼写错误
        common_patterns.insert("SELCT".to_string(), vec!["SELECT".to_string()]);
        common_patterns.insert("FORM".to_string(), vec!["FROM".to_string()]);
        common_patterns.insert("WHRE".to_string(), vec!["WHERE".to_string()]);
        common_patterns.insert("INSRT".to_string(), vec!["INSERT".to_string()]);
        common_patterns.insert("CREAT".to_string(), vec!["CREATE".to_string()]);
        
        Self {
            sql_keywords,
            common_patterns,
        }
    }

    /// 为错误消息生成诊断建议
    pub fn diagnose(&self, error_message: &str, context: Option<&DiagnosticContext>) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        // 检查拼写错误
        suggestions.extend(self.suggest_keyword_spelling(error_message));
        
        // 如果有上下文，提供更精确的建议
        if let Some(ctx) = context {
            suggestions.extend(self.suggest_table_names(error_message, &ctx.available_tables));
            suggestions.extend(self.suggest_column_names(error_message, &ctx.available_columns));
            suggestions.extend(self.suggest_syntax_fixes(error_message));
        }
        
        // 按置信度排序
        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        // 限制建议数量
        suggestions.truncate(5);
        
        suggestions
    }

    /// 格式化增强的错误消息
    pub fn format_enhanced_error(&self, original_error: &str, suggestions: &[Suggestion]) -> String {
        if suggestions.is_empty() {
            return original_error.to_string();
        }

        let mut result = format!("错误: {}\n", original_error);
        result.push_str("\n💡 建议:\n");
        
        for (i, suggestion) in suggestions.iter().enumerate() {
            let confidence_bar = "█".repeat((suggestion.confidence * 5.0) as usize);
            let type_label = match suggestion.suggestion_type {
                SuggestionType::KeywordSpelling => "拼写",
                SuggestionType::TableName => "表名", 
                SuggestionType::ColumnName => "列名",
                SuggestionType::SyntaxHint => "语法",
                SuggestionType::ExampleUsage => "示例",
            };
            
            result.push_str(&format!(
                "  {}. [{}] {} {}\n",
                i + 1,
                type_label,
                suggestion.text,
                confidence_bar
            ));
        }
        
        result
    }

    /// 建议关键字拼写
    fn suggest_keyword_spelling(&self, error_message: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        // 提取可能的关键字
        let words: Vec<&str> = error_message
            .split_whitespace()
            .collect();
            
        for word in words {
            let word_upper = word.to_uppercase();
            
            // 检查是否在常见错误模式中
            if let Some(corrections) = self.common_patterns.get(&word_upper) {
                for correction in corrections {
                    suggestions.push(Suggestion::new(
                        SuggestionType::KeywordSpelling,
                        format!("您是否想输入 '{}'?", correction),
                        0.9,
                    ));
                }
            } else {
                // 使用编辑距离查找相似关键字
                for keyword in &self.sql_keywords {
                    let distance = self.edit_distance(&word_upper, keyword);
                    if distance <= 2 && word_upper.len() > 2 {
                        let confidence = 1.0 - (distance as f64 / word_upper.len().max(keyword.len()) as f64);
                        if confidence > 0.6 {
                            suggestions.push(Suggestion::new(
                                SuggestionType::KeywordSpelling,
                                format!("您是否想输入 '{}'?", keyword),
                                confidence,
                            ));
                        }
                    }
                }
            }
        }
        
        suggestions
    }

    /// 建议表名
    fn suggest_table_names(&self, error_message: &str, available_tables: &[String]) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        if error_message.contains("table") || error_message.contains("表") {
            for table in available_tables.iter().take(3) {
                suggestions.push(Suggestion::new(
                    SuggestionType::TableName,
                    format!("可用表: {}", table),
                    0.8,
                ));
            }
        }
        
        suggestions
    }

    /// 建议列名
    fn suggest_column_names(&self, error_message: &str, available_columns: &[String]) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        if error_message.contains("column") || error_message.contains("列") {
            for column in available_columns.iter().take(3) {
                suggestions.push(Suggestion::new(
                    SuggestionType::ColumnName,
                    format!("可用列: {}", column),
                    0.8,
                ));
            }
        }
        
        suggestions
    }

    /// 建议语法修复
    fn suggest_syntax_fixes(&self, error_message: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        if error_message.contains("unexpected") || error_message.contains("意外") {
            suggestions.push(Suggestion::new(
                SuggestionType::SyntaxHint,
                "检查SQL语句的括号、引号和分号是否匹配".to_string(),
                0.7,
            ));
        }
        
        if error_message.contains("missing") || error_message.contains("缺少") {
            suggestions.push(Suggestion::new(
                SuggestionType::SyntaxHint,
                "可能缺少关键字、括号或分隔符".to_string(),
                0.7,
            ));
        }
        
        suggestions
    }

    /// 计算编辑距离（Levenshtein距离）
    fn edit_distance(&self, s1: &str, s2: &str) -> usize {
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        let len1 = s1_chars.len();
        let len2 = s2_chars.len();

        let mut dp = vec![vec![0; len2 + 1]; len1 + 1];

        // 初始化
        for i in 0..=len1 {
            dp[i][0] = i;
        }
        for j in 0..=len2 {
            dp[0][j] = j;
        }

        // 动态规划计算
        for i in 1..=len1 {
            for j in 1..=len2 {
                if s1_chars[i - 1] == s2_chars[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1];
                } else {
                    dp[i][j] = 1 + dp[i - 1][j].min(dp[i][j - 1]).min(dp[i - 1][j - 1]);
                }
            }
        }

        dp[len1][len2]
    }
}

/// 诊断上下文信息
#[derive(Debug, Clone)]
pub struct DiagnosticContext {
    /// 可用的表名
    pub available_tables: Vec<String>,
    /// 可用的列名
    pub available_columns: Vec<String>,
    /// 当前SQL位置信息
    pub position: Option<(usize, usize)>, // (行, 列)
}

impl DiagnosticContext {
    /// 创建新的诊断上下文
    pub fn new(available_tables: Vec<String>, available_columns: Vec<String>) -> Self {
        Self {
            available_tables,
            available_columns,
            position: None,
        }
    }

    /// 设置位置信息
    pub fn with_position(mut self, line: usize, column: usize) -> Self {
        self.position = Some((line, column));
        self
    }
}

impl Default for DiagnosticEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 错误诊断包装器 - 对外接口
pub fn enhance_error_message(
    error_message: &str,
    available_tables: Option<&[String]>,
    available_columns: Option<&[String]>,
) -> String {
    let engine = DiagnosticEngine::new();
    
    let context = if let (Some(tables), Some(columns)) = (available_tables, available_columns) {
        Some(DiagnosticContext::new(tables.to_vec(), columns.to_vec()))
    } else {
        None
    };
    
    let suggestions = engine.diagnose(error_message, context.as_ref());
    engine.format_enhanced_error(error_message, &suggestions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_spelling_suggestion() {
        let engine = DiagnosticEngine::new();
        let suggestions = engine.suggest_keyword_spelling("SELCT * FROM users");
        
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].text.contains("SELECT"));
    }

    #[test]
    fn test_edit_distance() {
        let engine = DiagnosticEngine::new();
        assert_eq!(engine.edit_distance("SELCT", "SELECT"), 1); // 插入E
        assert_eq!(engine.edit_distance("FROM", "FORM"), 2); // 交换R和O需要2步
        assert_eq!(engine.edit_distance("WHER", "WHERE"), 1); // 插入E
        assert_eq!(engine.edit_distance("", ""), 0);
        assert_eq!(engine.edit_distance("abc", ""), 3);
    }

    #[test]
    fn test_diagnose_with_context() {
        let engine = DiagnosticEngine::new();
        let context = DiagnosticContext::new(
            vec!["users".to_string(), "orders".to_string()],
            vec!["id".to_string(), "name".to_string(), "email".to_string()],
        );
        
        let suggestions = engine.diagnose("table not found", Some(&context));
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_format_enhanced_error() {
        let engine = DiagnosticEngine::new();
        let suggestions = vec![
            Suggestion::new(
                SuggestionType::KeywordSpelling,
                "您是否想输入 'SELECT'?".to_string(),
                0.9,
            ),
        ];
        
        let formatted = engine.format_enhanced_error("语法错误", &suggestions);
        assert!(formatted.contains("建议"));
        assert!(formatted.contains("SELECT"));
    }

    #[test]
    fn test_enhance_error_message() {
        let tables = vec!["users".to_string(), "orders".to_string()];
        let columns = vec!["id".to_string(), "name".to_string()];
        
        let enhanced = enhance_error_message(
            "SELCT * FROM nonexistent",
            Some(&tables),
            Some(&columns),
        );
        
        assert!(enhanced.contains("建议"));
    }
}