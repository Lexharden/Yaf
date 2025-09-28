# Guía de Estilo YAF

## Filosofía
Código claro, consistente y mínimo. Priorizar legibilidad sobre "cleverness".

## Reglas Generales
- Una instrucción por línea
- Evitar lógica compleja inline: usar funciones
- Nombres descriptivos en minúsculas con `_`

## Nombres
- Variables: `total_suma`, `indice`
- Funciones: `calcular_media`, `es_primo`

## Espacios y Sangría
- Indentación sugerida: 4 espacios (no obligatorio por el compilador)
- Espacios alrededor de operadores: `a + b * c`

## Bloques
```
if condicion {
    # cuerpo
} else {
    # otro
}
```

## Funciones
- Máximo sugerido: 25-30 líneas
- Usar retorno temprano para simplificar

## Strings
- Preferir mensajes claros: `print("Error: entrada invalida")`

## Organización de Archivo
Orden recomendado:
1. Comentario de encabezado (descripción)
2. Funciones generales
3. Funciones especializadas
4. Lógica principal

## Comentarios
- Explicar el "por qué", no el "qué"
```
# Evitamos division por cero
```

## Ejemplo
```
# Determina si un numero es primo
func es_primo(n: int) -> int {
    if n < 2 { return 0 }
    i = 2
    for i = 2; i < n; i = i + 1 {
        if n % i == 0 { return 0 }
    }
    return 1
}

print(es_primo(13))
```

---
La consistencia mejora la mantenibilidad y reduce errores.
