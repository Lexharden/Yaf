# YAF Language 🚀

> *"Tan legible como Python, tan sólido como Rust."*

YAF nace como un lenguaje que busca unir lo mejor de dos mundos: la robustez de Rust y la legibilidad de Python. El nombre surge tanto de su creador, Yafel, como del concepto de ser "Yet Another Future" en el mundo de los lenguajes modernos: accesible, seguro y eficiente.

## ✨ Visión
Un lenguaje pedagógico y experimental: claro para leer, preciso al fallar y eficiente al ejecutarse. Base ideal para aprender construcción de compiladores y evolucionar nuevas ideas.

## 🧱 Estado Actual (v0.1 prototipo)
- Sintaxis minimalista (inspirada en Python + C-style blocks)
- Tipos primitivos: `int`, `string`
- Funciones con parámetros tipados y retorno opcional
- Control de flujo: `if / else`, `for` (estilo C)
- Expresiones aritméticas y comparaciones
- Llamadas a funciones anidadas
- Librería estándar inicial: IO, math, strings (básico), colecciones simuladas
- Errores con línea y columna exacta (diagnóstico con caret)
- Backend LLVM funcional con optimizaciones básicas (constant folding)

## 🔜 En el Roadmap
- Tipo `bool` real
- Arrays nativos y estructuras compuestas
- Operadores lógicos (`&&`, `||`)
- Sistema de módulos / imports
- Backend WASM
- Mejor GC y valores dinámicos

## 🚀 Instalación
```bash
git clone https://github.com/Lexharden/Yaf.git
cd YafRust
cargo build --release
```
Binario: `./target/release/yaf` (no se versiona, repo limpio sólo mantiene código fuente y ejemplos `.yaf`).

## 🏁 Uso Básico
Archivo `hola.yaf`:
```yaf
print("Hola YAF!")
```

### Opción 1: Compilar y ejecutar en un paso
```bash
./target/release/yaf run hola.yaf
```

### Opción 2: Compilar y ejecutar por separado
```bash
./target/release/yaf compile hola.yaf
./hola
```

## 🛠 Comandos CLI Completos

### Comandos Principales
```bash
# 🚀 Compilar y ejecutar directamente (recomendado)
./target/release/yaf run archivo.yaf

# 🔧 Solo compilar
./target/release/yaf compile archivo.yaf

# ✅ Verificar sintaxis y tipos (sin compilar)
./target/release/yaf check archivo.yaf

# ℹ️  Información del compilador
./target/release/yaf info
```

### Opciones de Compilación
```bash
# Especificar backend
./target/release/yaf compile programa.yaf --backend llvm    # LLVM (por defecto)
./target/release/yaf compile programa.yaf --backend c      # C (máxima compatibilidad)

# Especificar archivo de salida
./target/release/yaf compile programa.yaf -o mi_programa

# Optimizaciones (0=ninguna, 1=básica, 2=agresiva, 3=máxima)
./target/release/yaf compile programa.yaf -O3

# Modo verbose (información detallada)
./target/release/yaf -v compile programa.yaf

# Generar archivos intermedios
./target/release/yaf compile programa.yaf --emit-llvm     # Genera .ll
./target/release/yaf compile programa.yaf --emit-asm     # Genera ensamblador
./target/release/yaf compile programa.yaf --keep-temps   # Mantener archivos temporales

# Información de debug
./target/release/yaf compile programa.yaf --debug
```

### Verificación y Desarrollo
```bash
# Verificar sintaxis con información detallada
./target/release/yaf -v check archivo.yaf

# Verificar todos los ejemplos
for f in examples/*.yaf; do ./target/release/yaf check "$f"; done

# Ejecutar con argumentos
./target/release/yaf run programa.yaf -- arg1 arg2 arg3
```

### Comandos Experimentales
```bash
# Formatear código (próximamente)
./target/release/yaf format archivo.yaf

# REPL interactivo (próximamente)
./target/release/yaf repl

# Language Server Protocol (próximamente)
./target/release/yaf lsp
```

### Ayuda y Opciones Globales
```bash
# Mostrar ayuda general
./target/release/yaf --help

# Ayuda de comando específico
./target/release/yaf compile --help
./target/release/yaf run --help

# Versión
./target/release/yaf --version

# Arquitectura objetivo
./target/release/yaf compile programa.yaf --target x86_64
./target/release/yaf compile programa.yaf --target native
```

## 🧪 Ejemplos
La carpeta `examples/` contiene programas progresivos:
1. Hola mundo
2. Variables / I/O
3. Condicionales
4. Funciones (recursión, utilidades)
5. Arrays simulados y loops
6. Funciones matemáticas
7. Strings
8. Algoritmos
9. Estructuras de datos simuladas
10. Mini proyecto (inventario)
11. Juego sencillo
12. Entrada múltiple
13. **Benchmarks** (medición de tiempo con `now_millis()`)

### Ejecutar ejemplos:
```bash
# Verificar sintaxis primero
./target/release/yaf check examples/01_hello_world.yaf

# Ejecutar directamente (recomendado)
./target/release/yaf run examples/01_hello_world.yaf

# Compilar y ejecutar por separado
./target/release/yaf compile examples/01_hello_world.yaf
./examples/01_hello_world

# Ejemplos específicos con optimizaciones
./target/release/yaf run examples/13_becnhmark_time.yaf -O3    # Benchmarks optimizados
./target/release/yaf -v run examples/08_algorithms.yaf        # Algoritmos con verbose

# Probar diferentes backends
./target/release/yaf run examples/04_functions.yaf --backend llvm
./target/release/yaf run examples/04_functions.yaf --backend c
```

## 🧬 Sintaxis Esencial
### Función
```yaf
func sumar(a: int, b: int) -> int {
    return a + b
}
print(sumar(3, 4))
```
### If / Else
```yaf
x = 10
if x < 15 {
    print("Menor")
} else {
    print("Mayor o igual")
}
```
### For
```yaf
for i = 0; i < 5; i = i + 1 {
    print(i)
}
```
### Recursión
```yaf
func fact(n: int) -> int {
    if n == 0 { return 1 }
    return n * fact(n - 1)
}
print(fact(5))
```

## 🧰 Librería Estándar (subset)
| Categoria | Funciones |
|-----------|-----------|
| IO        | `print(...)`, `input()` |
| Math      | `pow`, `factorial`, `gcd`, `fib` |
| String    | `concat`, `equals` |
| Time      | `now()`, `now_millis()` |
| Demo col. | utilidades en ejemplos 09 |

### 🕰 Benchmarking y Medición de Tiempo
```yaf
# Medir tiempo de ejecución
start = now_millis()
# ... tu código aquí ...
end = now_millis()
print("Tiempo:", end - start, "ms")
```

## 🛡 Diagnóstico de Errores
Ejemplo de string no cerrado:
```
Error: String no cerrado
 --> archivo.yaf:12:10
  |
12 | print("Hola
  |        ^ inicio del string
```

## � Casos de Uso Prácticos

### Desarrollo Iterativo
```bash
# 1. Verificar sintaxis mientras escribes código
./target/release/yaf check mi_algoritmo.yaf

# 2. Probar rápidamente sin generar ejecutable
./target/release/yaf run mi_algoritmo.yaf

# 3. Compilar versión optimizada final
./target/release/yaf compile mi_algoritmo.yaf -O3 -o release_final
```

### Benchmarking y Optimización
```bash
# Comparar backends
time ./target/release/yaf run examples/13_becnhmark_time.yaf --backend llvm
time ./target/release/yaf run examples/13_becnhmark_time.yaf --backend c

# Probar diferentes niveles de optimización
./target/release/yaf run examples/08_algorithms.yaf -O0  # Sin optimización
./target/release/yaf run examples/08_algorithms.yaf -O3  # Máxima optimización
```

### Debug y Análisis
```bash
# Información detallada del proceso de compilación
./target/release/yaf -v compile programa.yaf

# Generar LLVM IR para análisis
./target/release/yaf compile programa.yaf --emit-llvm
cat programa.ll  # Ver código intermedio generado

# Compilar con información de debug
./target/release/yaf compile programa.yaf --debug
```

### Automatización y CI/CD
```bash
# Verificar todos los archivos en un proyecto
find . -name "*.yaf" -exec ./target/release/yaf check {} \;

# Script de build automatizado
for f in src/*.yaf; do
    echo "Building $f..."
    ./target/release/yaf compile "$f" -O2 || exit 1
done
```

## �🗂 Documentación
Todo en `docs/`:
- `ARCHITECTURE.md`
- `LANGUAGE_SPEC.md`
- `QUICKSTART.md`
- `STYLE_GUIDE.md`
- `TUTORIAL.md`
- `docs/README.md` (índice)

## 🤝 Contribuir
1. Abre un issue con propuesta
2. Agrega ejemplo si introduces sintaxis nueva
3. Mantén cambios mínimos y bien justificados

## 📄 Licencia
MIT — ver `LICENSE`.

## 👤 Autor
Creador: **Yafel Garcia** (Yafel)  
"Yet Another Future" es una visión de accesibilidad, seguridad y eficiencia.

---
YAF: *Yet Another Future* — construir, aprender, iterar.