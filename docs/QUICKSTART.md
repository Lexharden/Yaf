# Guía Rápida de YAF

## 1. Instalar (compilar el compilador)
```
cargo build --release
```
El binario quedará en `target/release/yaf` (no incluido en repo limpio).

## 2. Crear un Programa
Archivo: `hola.yaf`
```
print("Hola YAF!")
```

## 3. Compilar y Ejecutar

### Opción rápida (recomendada):
```
./target/release/yaf run hola.yaf
```

### Opción paso a paso:
```
./target/release/yaf compile hola.yaf
./hola
```

## 5. Funciones
```
func saludar(nombre: string) {
    print("Hola ", nombre, "!")
}

saludar("Mundo")
```

## 6. Variables y Expresiones
```
a = 10
b = 20
print("Suma:", a + b)
```

## 7. Control de Flujo
```
if a < b { print("a menor") } else { print("a mayor o igual") }

for i = 0; i < 3; i = i + 1 {
    print("i =", i)
}
```

## 8. Recursión
```
func fact(n: int) -> int {
    if n == 0 { return 1 }
    return n * fact(n - 1)
}
print(fact(5))
```

## 9. Entrada de Usuario
```
print("Tu nombre:")
nombre = input()
print("Hola", nombre)
```

## 10. Comandos Disponibles

### Comandos Esenciales
```bash
# Compilar y ejecutar directo
./target/release/yaf run archivo.yaf

# Solo compilar  
./target/release/yaf compile archivo.yaf

# Verificar sintaxis (sin compilar)
./target/release/yaf check archivo.yaf

# Información del compilador
./target/release/yaf info
```

### Opciones Útiles
```bash
# Verbose (información detallada)
./target/release/yaf -v run archivo.yaf

# Optimización máxima
./target/release/yaf run archivo.yaf -O3

# Backend específico
./target/release/yaf run archivo.yaf --backend c

# Ver ayuda
./target/release/yaf --help
./target/release/yaf compile --help
```

## 11. Consultar Más
Revisa:
- `docs/LANGUAGE_SPEC.md`
- `examples/` (especialmente `13_pruebas.yaf` para benchmarks)
- `docs/TUTORIAL.md`

---
YAF es ideal para aprender compiladores y experimentar con diseño de lenguajes.
