# Tutorial Paso a Paso de YAF

Bienvenido a YAF: "Tan legible como Python, tan sólido co## 11. Benchmarking y Tiempo
```yaf
# Medir rendimiento de algoritmos
start = now_millis()
resultado = fib(25)
end = now_millis()
print("Fibonacci(25):", resultado)
print("Tiempo:", end - start, "ms")
```

## 12. Próximos Pasos
- Lee `LANGUAGE_SPEC.md`
- Explora `examples/` (especialmente 13_pruebas.yaf)
- Extiende el runtimeust." Este tutorial te guía desde cero hasta crear un mini proyecto.

## 1. Hola Mundo
```
print("Hola Mundo")
```
Compila y ejecuta como se explica en `QUICKSTART.md`.

## 2. Variables y Operaciones
```
a = 5
b = 7
print("Resultado:", a * b + 3)
```

## 3. Funciones Básicas
```
func cuadrado(x: int) -> int { return x * x }
print(cuadrado(9))
```

## 4. Condicionales
```
nota = 85
if nota >= 60 {
    print("Aprobado")
} else {
    print("Reprobado")
}
```

## 5. Bucles
```
for i = 0; i < 5; i = i + 1 {
    print("i =", i)
}
```

## 6. Recursión
```
func fib(n: int) -> int {
    if n < 2 { return n }
    return fib(n - 1) + fib(n - 2)
}
print(fib(8))
```

## 7. Entrada del Usuario
```
print("Tu nombre:")
nombre = input()
print("Hola", nombre)
```

## 8. Algoritmia
```
func es_primo(n: int) -> int {
    if n < 2 { return 0 }
    i = 2
    for i = 2; i < n; i = i + 1 {
        if n % i == 0 { return 0 }
    }
    return 1
}
print(es_primo(29))
```

## 9. Proyecto Mini: Inventario
```
func mostrar_item(nombre: string, precio: int, stock: int) {
    print(nombre, " - $", precio, " - Stock:", stock)
}

print("=== INVENTARIO ===")
mostrar_item("Laptop", 1200, 5)
mostrar_item("Mouse", 25, 50)
```

## 10. Manejo de Errores (Experimento)
Prueba quitar una comilla de un string y observa el mensaje preciso del compilador.

## 11. Buenas Prácticas
- Divide lógica en funciones
- Usa nombres descriptivos
- Evita duplicación

## 12. Próximos Pasos
- Lee `LANGUAGE_SPEC.md`
- Explora `examples/`
- Extiende el runtime

---
¡Listo! Ya dominas lo esencial de YAF.
