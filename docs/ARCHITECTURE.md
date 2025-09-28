# Arquitectura de YAF

YAF (Yet Another Future) combina la robustez de Rust con la legibilidad de Python. Su arquitectura está organizada en capas claras para facilitar mantenimiento, extensibilidad y rendimiento.

## Objetivos de Diseño
- Simplicidad sintáctica
- Tipado estático claro y predecible
- Errores con localización precisa (línea/columna)
- Backend modular (LLVM / C / futuro WASM)
- Ejecución eficiente con optimizaciones básicas
- Base preparada para un recolector de basura incremental

## Flujo de Compilación
```
Fuente (.yaf) -> Lexer -> Tokens -> Parser -> AST -> TypeChecker -> IR interno -> Backend (LLVM/C) -> Código máquina / binario
```

## Componentes Principales

### 1. Lexer (`src/core/lexer.rs`)
Responsable de convertir el texto fuente en tokens. Funcionalidades clave:
- Soporte para palabras clave: `func`, `function`, `if`, `else`, `for`, `return`
- Manejo de enteros y strings (con detección de strings no cerrados)
- Generación de `LexErrorWithPosition` con línea y columna exactas
- Tokens enriquecidos con `TokenInfo { token, line, column }`

### 2. Parser (`src/core/parser.rs`)
Transforma tokens en un Árbol de Sintaxis Abstracta (AST). Características:
- Declaraciones de funciones y variables
- Expresiones aritméticas, booleanas y llamadas a funciones
- Estructuras de control: if/else y for
- Errores con contexto usando `ParseErrorWithPosition`

### 3. AST (`src/core/ast.rs`)
Modelo estructural intermedio. Provee variantes para:
- Expresiones: literales, binarias, unarias, llamadas
- Sentencias: declaraciones, asignaciones, retorno, bloque, condicional, bucle for

### 4. Comprobación de Tipos (`src/core/typechecker.rs`)
Valida consistencia de tipos y prepara metadatos para backend.

### 5. Runtime (`runtime/` y `src/runtime/*`)
Incluye:
- Representación de valores en ejecución
- (WIP) Recolector de basura (`gc.rs`)
- Gestión de memoria (`memory.rs`)

### 6. Librerías Estándar (`src/libs/`)
Módulos embebidos accesibles vía funciones predefinidas:
- `io`: impresión y lectura básica
- `math`: potencia, factorial, mcd, fibonacci, etc.
- `string`: concatenación y comparación
- `collections`: estructuras simuladas (arrays dinámicos, pila, cola)
- `time`, `net` (bases para expansión futura)

### 7. Backends (`src/backend/`)
- `llvm.rs`: Generación de IR + optimizaciones sencillas
- `c.rs`: (plantilla inicial para salida C)
- Diseño extensible para agregar WASM / JIT

### 8. Diagnóstico (`src/diagnostics.rs`)
Responsable de formatear y mostrar errores:
- Línea y columna con caret `^`
- Extracto de código fuente
- Mensajes claros y consistentes

## Manejo de Errores
Tipos diferenciados:
- Léxicos: strings no cerrados, caracteres inválidos
- Sintácticos: tokens inesperados, estructura incompleta
- Semánticos (futuro): tipos incompatibles, funciones no declaradas

## Optimizaciones LLVM Implementadas
- Constant folding básico
- Eliminación de código muerto (parcial)
- Preparado para inlining y loop unrolling futuro

## Ejecución de Ejemplos
Todos los ejemplos en `examples/` se compilan a binarios auto-contenidos.

## Futuras Extensiones
- Sistema de módulos/imports
- Tipos compuestos (struct, array real, map)
- Inferencia parcial de tipos
- Strings UTF-8 avanzadas
- Backend WASM
- GC generacional

---
YAF busca ser una base sólida para experimentar con diseño de lenguajes modernos sin sacrificar claridad.
