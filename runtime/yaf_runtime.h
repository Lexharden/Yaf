#ifndef YAF_RUNTIME_H
#define YAF_RUNTIME_H

#include <stdint.h>
#include <stdbool.h>

// YafValue structure matching Rust implementation
typedef struct {
    uint8_t tag;
    union {
        int64_t int_val;
        double float_val;
        char* string_val;
        bool bool_val;
        void* array_val;
    } value;
} YafValue;

// Value type tags
#define YAF_INT    0
#define YAF_FLOAT  1
#define YAF_STRING 2 
#define YAF_BOOL   3
#define YAF_ARRAY  4

// Runtime functions
YafValue yaf_make_int(int64_t value);
YafValue yaf_make_float(double value);
YafValue yaf_make_string(const char* value);
YafValue yaf_make_bool(int value);
YafValue yaf_make_void(void);

// Type conversion functions
YafValue yaf_value_to_string(YafValue value);
YafValue yaf_value_to_int(YafValue value);
YafValue yaf_value_to_float(YafValue value);
void yaf_free_value(YafValue* value);
void yaf_print_value(YafValue value);
void yaf_print_value_no_newline(YafValue value);

// Math functions
YafValue yaf_math_abs(YafValue value);
YafValue yaf_math_max(YafValue a, YafValue b);
YafValue yaf_math_min(YafValue a, YafValue b);
YafValue yaf_math_pow(YafValue base, YafValue exp);

// String functions
YafValue yaf_string_length(YafValue s);
YafValue yaf_string_upper(YafValue s);
YafValue yaf_string_lower(YafValue s);
YafValue yaf_string_concat(YafValue a, YafValue b);

// I/O functions
YafValue yaf_io_read_file(YafValue path);
YafValue yaf_io_write_file(YafValue path, YafValue content);
YafValue yaf_io_file_exists(YafValue path);
YafValue yaf_io_input(void);
YafValue yaf_io_input_prompt(YafValue prompt);

// Time functions
YafValue yaf_time_now(void);
YafValue yaf_time_now_millis(void);
YafValue yaf_time_sleep(YafValue seconds);

// Type conversion functions (enhanced)
YafValue yaf_string_to_int(YafValue s);
YafValue yaf_int_to_string(YafValue i);

#endif // YAF_RUNTIME_H