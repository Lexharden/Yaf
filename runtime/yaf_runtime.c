#include "yaf_runtime.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <math.h>
#include <unistd.h>
#include <time.h>
#include <sys/time.h>
#include <ctype.h>

// Helper function to check value types
static void validate_type(YafValue val, uint8_t expected_type, const char* func_name) {
    if (val.tag != expected_type) {
        fprintf(stderr, "Runtime error in %s: expected type %d, got %d\n", 
                func_name, expected_type, val.tag);
        exit(1);
    }
}

// Value construction functions
YafValue yaf_make_int(int64_t value) {
    YafValue val;
    val.tag = YAF_INT;
    val.value.int_val = value;
    return val;
}

YafValue yaf_make_float(double value) {
    YafValue val;
    val.tag = YAF_FLOAT;
    val.value.float_val = value;
    return val;
}

YafValue yaf_make_string(const char* value) {
    YafValue val;
    val.tag = YAF_STRING;
    val.value.string_val = strdup(value ? value : "");
    return val;
}

YafValue yaf_make_bool(int value) {
    YafValue val;
    val.tag = YAF_BOOL;
    val.value.bool_val = value != 0;
    return val;
}

YafValue yaf_make_void(void) {
    YafValue val;
    val.tag = YAF_INT;  // Use INT with value 0 for void
    val.value.int_val = 0;
    return val;
}

// Type conversion functions
YafValue yaf_value_to_string(YafValue value) {
    char buffer[64];
    switch (value.tag) {
        case YAF_INT:
            snprintf(buffer, sizeof(buffer), "%lld", value.value.int_val);
            return yaf_make_string(buffer);
        case YAF_FLOAT:
            snprintf(buffer, sizeof(buffer), "%g", value.value.float_val);
            return yaf_make_string(buffer);
        case YAF_STRING:
            return yaf_make_string(value.value.string_val);
        case YAF_BOOL:
            return yaf_make_string(value.value.bool_val ? "true" : "false");
        default:
            return yaf_make_string("unknown");
    }
}

YafValue yaf_value_to_int(YafValue value) {
    switch (value.tag) {
        case YAF_INT:
            return value;
        case YAF_FLOAT:
            return yaf_make_int((int64_t)value.value.float_val);
        case YAF_STRING:
            return yaf_make_int(strtoll(value.value.string_val, NULL, 10));
        case YAF_BOOL:
            return yaf_make_int(value.value.bool_val ? 1 : 0);
        default:
            return yaf_make_int(0);
    }
}

YafValue yaf_value_to_float(YafValue value) {
    switch (value.tag) {
        case YAF_INT:
            return yaf_make_float((double)value.value.int_val);
        case YAF_FLOAT:
            return value;
        case YAF_STRING:
            return yaf_make_float(strtod(value.value.string_val, NULL));
        case YAF_BOOL:
            return yaf_make_float(value.value.bool_val ? 1.0 : 0.0);
        default:
            return yaf_make_float(0.0);
    }
}

void yaf_free_value(YafValue* value) {
    if (value->tag == YAF_STRING && value->value.string_val) {
        free(value->value.string_val);
        value->value.string_val = NULL;
    }
}

void yaf_print_value(YafValue value) {
    yaf_print_value_no_newline(value);
    printf("\n");
}

void yaf_print_value_no_newline(YafValue value) {
    switch (value.tag) {
        case YAF_INT:
            printf("%lld", value.value.int_val);
            break;
        case YAF_FLOAT:
            printf("%g", value.value.float_val);
            break;
        case YAF_STRING:
            printf("%s", value.value.string_val ? value.value.string_val : "");
            break;
        case YAF_BOOL:
            printf("%s", value.value.bool_val ? "true" : "false");
            break;
        default:
            printf("unknown");
            break;
    }
}

// Math functions
YafValue yaf_math_abs(YafValue value) {
    switch (value.tag) {
        case YAF_INT:
            return yaf_make_int(llabs(value.value.int_val));
        case YAF_FLOAT:
            return yaf_make_float(fabs(value.value.float_val));
        default:
            return yaf_make_int(0);
    }
}

YafValue yaf_math_max(YafValue a, YafValue b) {
    if (a.tag == YAF_INT && b.tag == YAF_INT) {
        return yaf_make_int(a.value.int_val > b.value.int_val ? a.value.int_val : b.value.int_val);
    } else if (a.tag == YAF_FLOAT || b.tag == YAF_FLOAT) {
        double a_val = (a.tag == YAF_FLOAT) ? a.value.float_val : (double)a.value.int_val;
        double b_val = (b.tag == YAF_FLOAT) ? b.value.float_val : (double)b.value.int_val;
        return yaf_make_float(a_val > b_val ? a_val : b_val);
    }
    return yaf_make_int(0);
}

YafValue yaf_math_min(YafValue a, YafValue b) {
    if (a.tag == YAF_INT && b.tag == YAF_INT) {
        return yaf_make_int(a.value.int_val < b.value.int_val ? a.value.int_val : b.value.int_val);
    } else if (a.tag == YAF_FLOAT || b.tag == YAF_FLOAT) {
        double a_val = (a.tag == YAF_FLOAT) ? a.value.float_val : (double)a.value.int_val;
        double b_val = (b.tag == YAF_FLOAT) ? b.value.float_val : (double)b.value.int_val;
        return yaf_make_float(a_val < b_val ? a_val : b_val);
    }
    return yaf_make_int(0);
}

YafValue yaf_math_pow(YafValue base, YafValue exp) {
    double base_val = (base.tag == YAF_FLOAT) ? base.value.float_val : (double)base.value.int_val;
    double exp_val = (exp.tag == YAF_FLOAT) ? exp.value.float_val : (double)exp.value.int_val;
    return yaf_make_float(pow(base_val, exp_val));
}

// String functions
YafValue yaf_string_length(YafValue s) {
    validate_type(s, YAF_STRING, "string_length");
    return yaf_make_int(strlen(s.value.string_val ? s.value.string_val : ""));
}

YafValue yaf_string_upper(YafValue s) {
    validate_type(s, YAF_STRING, "string_upper");
    const char* input = s.value.string_val ? s.value.string_val : "";
    char* result = malloc(strlen(input) + 1);
    for (size_t i = 0; input[i]; i++) {
        result[i] = toupper(input[i]);
    }
    result[strlen(input)] = '\0';
    YafValue val = yaf_make_string(result);
    free(result);
    return val;
}

YafValue yaf_string_lower(YafValue s) {
    validate_type(s, YAF_STRING, "string_lower");
    const char* input = s.value.string_val ? s.value.string_val : "";
    char* result = malloc(strlen(input) + 1);
    for (size_t i = 0; input[i]; i++) {
        result[i] = tolower(input[i]);
    }
    result[strlen(input)] = '\0';
    YafValue val = yaf_make_string(result);
    free(result);
    return val;
}

YafValue yaf_string_concat(YafValue a, YafValue b) {
    const char* a_str = (a.tag == YAF_STRING && a.value.string_val) ? a.value.string_val : "";
    const char* b_str = (b.tag == YAF_STRING && b.value.string_val) ? b.value.string_val : "";
    
    size_t total_len = strlen(a_str) + strlen(b_str) + 1;
    char* result = malloc(total_len);
    strcpy(result, a_str);
    strcat(result, b_str);
    
    YafValue val = yaf_make_string(result);
    free(result);
    return val;
}

// I/O functions
YafValue yaf_io_read_file(YafValue path) {
    validate_type(path, YAF_STRING, "read_file");
    const char* filepath = path.value.string_val ? path.value.string_val : "";
    
    FILE* file = fopen(filepath, "r");
    if (!file) {
        return yaf_make_string("");
    }
    
    fseek(file, 0, SEEK_END);
    long length = ftell(file);
    fseek(file, 0, SEEK_SET);
    
    char* content = malloc(length + 1);
    fread(content, 1, length, file);
    content[length] = '\0';
    fclose(file);
    
    YafValue val = yaf_make_string(content);
    free(content);
    return val;
}

YafValue yaf_io_write_file(YafValue path, YafValue content) {
    validate_type(path, YAF_STRING, "write_file");
    validate_type(content, YAF_STRING, "write_file");
    
    const char* filepath = path.value.string_val ? path.value.string_val : "";
    const char* data = content.value.string_val ? content.value.string_val : "";
    
    FILE* file = fopen(filepath, "w");
    if (!file) {
        return yaf_make_bool(0);
    }
    
    fputs(data, file);
    fclose(file);
    return yaf_make_bool(1);
}

YafValue yaf_io_file_exists(YafValue path) {
    validate_type(path, YAF_STRING, "file_exists");
    const char* filepath = path.value.string_val ? path.value.string_val : "";
    return yaf_make_bool(access(filepath, F_OK) == 0);
}

YafValue yaf_io_input(void) {
    char buffer[1024];  // Buffer fijo de 1024 caracteres
    
    if (fgets(buffer, sizeof(buffer), stdin) == NULL) {
        return yaf_make_string("");
    }
    
    // Remove trailing newline if present
    size_t len = strlen(buffer);
    if (len > 0 && buffer[len-1] == '\n') {
        buffer[len-1] = '\0';
        len--;
    }
    // Remove trailing carriage return if present (Windows compatibility)
    if (len > 0 && buffer[len-1] == '\r') {
        buffer[len-1] = '\0';
    }
    
    return yaf_make_string(buffer);
}

YafValue yaf_io_input_prompt(YafValue prompt) {
    validate_type(prompt, YAF_STRING, "input_prompt");
    const char* prompt_str = prompt.value.string_val ? prompt.value.string_val : "";
    
    // Print prompt without newline
    printf("%s", prompt_str);
    fflush(stdout);
    
    return yaf_io_input();
}

// Time functions  
YafValue yaf_time_now(void) {
    return yaf_make_int(time(NULL));
}

YafValue yaf_time_now_millis(void) {
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return yaf_make_int(tv.tv_sec * 1000 + tv.tv_usec / 1000);
}

YafValue yaf_time_sleep(YafValue seconds) {
    int sec = (seconds.tag == YAF_INT) ? seconds.value.int_val : 
              (int)seconds.value.float_val;
    sleep(sec);
    return yaf_make_bool(1);
}

// Type conversion functions (enhanced)
YafValue yaf_string_to_int(YafValue s) {
    validate_type(s, YAF_STRING, "string_to_int");
    const char* str = s.value.string_val ? s.value.string_val : "";
    
    // Parse integer from string
    char* endptr;
    long long result = strtoll(str, &endptr, 10);
    
    // Check if conversion was successful
    if (endptr == str || *endptr != '\0') {
        return yaf_make_int(0);  // Return 0 for invalid strings
    }
    
    return yaf_make_int(result);
}

YafValue yaf_int_to_string(YafValue i) {
    validate_type(i, YAF_INT, "int_to_string");
    
    char buffer[32];
    snprintf(buffer, sizeof(buffer), "%lld", i.value.int_val);
    return yaf_make_string(buffer);
}

// GC functions - simple implementations
void yaf_gc_collect(void) {
    // Simple no-op for now
}

void yaf_gc_final_cleanup(void) {
    // Simple no-op for now  
}