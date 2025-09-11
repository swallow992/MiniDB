# Fix remaining SemanticError position field issues
$file = "src\sql\analyzer.rs"
$content = Get-Content $file -Raw

# Replace patterns for missing position fields
$patterns = @{
    'SemanticError::TableNotFound \{\s*table: ([^,}]+),?\s*\}'                                          = 'SemanticError::TableNotFound { table: $1, position: None }';
    'SemanticError::ColumnNotFound \{\s*table: ([^,]+),\s*column: ([^,}]+),?\s*\}'                      = 'SemanticError::ColumnNotFound { table: $1, column: $2, position: None }';
    'SemanticError::TypeMismatch \{\s*expected: ([^,]+),\s*found: ([^,}]+),?\s*\}'                      = 'SemanticError::TypeMismatch { expected: $1, found: $2, position: None }';
    'SemanticError::InsertColumnMismatch \{\s*expected: ([^,]+),\s*actual: ([^,}]+),?\s*\}'             = 'SemanticError::InsertColumnMismatch { expected: $1, actual: $2, position: None }';
    'SemanticError::NullConstraintViolation \{\s*column: ([^,}]+),?\s*\}'                               = 'SemanticError::NullConstraintViolation { column: $1, position: None }';
    'SemanticError::AmbiguousColumn \{\s*column: ([^,}]+),?\s*\}'                                       = 'SemanticError::AmbiguousColumn { column: $1, position: None }';
    'SemanticError::InvalidBinaryOperation \{\s*op: ([^,]+),\s*left: ([^,]+),\s*right: ([^,}]+),?\s*\}' = 'SemanticError::InvalidBinaryOperation { op: $1, left: $2, right: $3, position: None }';
    'SemanticError::InvalidUnaryOperation \{\s*op: ([^,]+),\s*operand: ([^,}]+),?\s*\}'                 = 'SemanticError::InvalidUnaryOperation { op: $1, operand: $2, position: None }'
}

foreach ($pattern in $patterns.Keys) {
    $replacement = $patterns[$pattern]
    $content = $content -replace $pattern, $replacement
}

$content | Set-Content $file
Write-Host "Fixed SemanticError position fields"
