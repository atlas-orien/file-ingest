#pragma once

#ifdef __cplusplus
extern "C" {
#endif

int file_ingest_md_convert(
    const char* file_type,
    const char* input_path,
    const char* output_dir,
    char** out_path,
    char** out_err);

void file_ingest_md_free(char* ptr);

#ifdef __cplusplus
}
#endif
