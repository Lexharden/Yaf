# Especificación del Lenguaje YAF

Versión: 0.1 (Prototipo)

## Filosofía
"Tan legible como Python, tan sólido como Rust." — Sintaxis clara, tipado explícito, ejecución eficiente.

## Tipos Primitivos
- `int` (enteros con signo)
- `string`
- (Futuro) `bool`, `float`

## Comentarios
Líneas que comienzan con `#`.
```
# Esto es un comentario
```

## Declaración de Funciones
```
func nombre(param1: int, param2: string) {
    # cuerpo
}

func sumar(a: int, b: int) -> int {
    return a + b
}
```
- Palabras clave válidas: `func` o `function`
- Retorno opcional con `-> tipo`

## Variables
Asignación directa (no existe `let`):
```
x = 10
nombre = "Veri"
```

## Expresiones
Soportadas: `+ - * / %`, comparación `== != < > <= >=`, anidación con paréntesis.

## Strings
- Delimitadas por comillas dobles
- Concatenación usando coma en `print` o con lógica futura

## Sentencias
### Bloques
```
{
  x = 1
  y = 2
}
```

### If / Else
```
if x < 10 {
    print("Menor que 10")
} else {
    print("Mayor o igual a 10")
}
```

### For
Forma actual (contador simple):
```
for i = 0; i < 5; i = i + 1 {
    print(i)
}
```

## Llamadas a Funciones
```
resultado = sumar(3, 4)
print("Resultado:", resultado)
```

## Return
Debe estar dentro de una función:
```
func factorial(n: int) -> int {
    if n == 0 {
        return 1
    }
    return n * factorial(n - 1)
}
```

## Librería Estándar (parcial)
### IO
- `print(val1, val2, ...)`
- `input()` -> string

### Math
- `pow(base, exp)`
- `factorial(n)`
- `gcd(a, b)`
- `fib(n)`

### Strings
- `concat(a, b)`
- `equals(a, b)` (1 si igual, 0 si distinto)

### Time
- `now()` -> segundos desde Unix epoch
- `now_millis()` -> milisegundos desde Unix epoch

### Collections (simuladas)
Ejemplos demostrativos en `09_data_structures.yaf`.

## Errores y Diagnóstico
Formato típico:
```
Error: String no cerrado
 --> archivo.yaf:107:27
  |
107 | print("Hola
  |        ^ inicio del string
```

## Limitaciones Actuales
- No hay tipos booleanos explícitos (se usan enteros 0/1)
- No hay operadores lógicos `&&` / `||` todavía
- Sin arrays reales (simulados manualmente)
- Sin módulos/imports

## Ejemplo Completo
```
func es_primo(n: int) -> int {
    if n < 2 { return 0 }
    i = 2
    for i = 2; i < n; i = i + 1 {
        if n % i == 0 { return 0 }
    }
    return 1
}

print("13 primo?", es_primo(13))
```

---
Esta especificación crecerá a medida que evolucione el lenguaje.
