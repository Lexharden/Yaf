# Changelog

## [0.1.0] - 2025-09-28

### 🎉 Primera Release Oficial de YAF

**YAF (Yet Another Future)** - "Tan legible como Python, tan sólido como Rust."

### ✨ Características Principales

#### 🦾 Compilador
- **Backend LLVM** completo con optimizaciones (constant folding)
- **Backend C** alternativo para máxima compatibilidad
- **Sistema de errores** con línea y columna exacta
- **Diagnóstico profesional** con caret apuntando al error

#### 🔤 Lenguaje
- **Sintaxis clara** inspirada en Python
- **Tipado estático** para seguridad
- **Funciones** con parámetros tipados y retorno opcional
- **Control de flujo**: `if/else`, `for` loops
- **Recursión** completa y eficiente
- **Expresiones** aritméticas y booleanas

#### 📚 Librería Estándar
- **I/O**: `print()`, `input()`
- **Matemáticas**: `pow()`, `factorial()`, `gcd()`, `fib()`
- **Strings**: `concat()`, `equals()`
- **Tiempo**: `now()`, `now_millis()` para benchmarking
- **Conversiones**: `string_to_int()`, `int_to_string()`

#### 🛠 Herramientas
- **Comando `run`**: Compila y ejecuta en un paso
- **Comando `compile`**: Solo compilación
- **Optimizaciones LLVM** automáticas
- **Benchmarking** integrado con medición de tiempo

### 📦 Ejemplos Incluidos

13 ejemplos progresivos que cubren todas las características:

1. **Hello World** - Impresión básica
2. **Variables** - Calculadora interactiva
3. **Condicionales** - Sistema de verificación
4. **Funciones** - Recursión y utilidades
5. **Arrays/Loops** - Manipulación de datos
6. **Matemáticas** - Funciones avanzadas
7. **Strings** - Procesamiento de texto
8. **Algoritmos** - Búsqueda, ordenamiento, primos
9. **Estructuras** - Pilas, colas, listas simuladas
10. **Proyecto** - Sistema de inventario completo
11. **Juego** - RPG interactivo
12. **Input** - Entrada múltiple
13. **Benchmarks** - Medición de rendimiento

### 🚀 Rendimiento

- **Compilación rápida**: 6-12 segundos para ejemplos
- **Ejecución eficiente**: Código nativo optimizado
- **Benchmarks**: 8,713 primos (2-90,000) en 2.9s

### 📖 Documentación Completa

- **README.md** - Guía principal
- **docs/ARCHITECTURE.md** - Diseño interno
- **docs/LANGUAGE_SPEC.md** - Especificación formal
- **docs/QUICKSTART.md** - Inicio rápido
- **docs/TUTORIAL.md** - Aprendizaje paso a paso
- **docs/STYLE_GUIDE.md** - Convenciones de código

### 🔧 Instalación

```bash
git clone https://github.com/Lexharden/Yaf.git
cd Yaf
cargo build --release
./target/release/yaf run examples/01_hello_world.yaf
```

### 👥 Créditos

- **Creador**: Yafel Garcia (Yafel)
- **Visión**: Accesibilidad, seguridad y eficiencia en lenguajes de programación

---

¡Bienvenidos al futuro de la programación con YAF! 🌟