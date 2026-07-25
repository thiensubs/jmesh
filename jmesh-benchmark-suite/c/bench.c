/*
 * bench.c
 * Reproducible benchmark for SQLite import/export performance.
 * Measures: wall time, CPU time (user+sys), peak memory (RSS).
 *
 * Compile: gcc -O3 -o bench bench.c -lsqlite3 -lm
 * Run:     ./bench
 * Output:  results.json (machine-readable)
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <sys/resource.h>
#include <sys/time.h>
#include <unistd.h>
#include <sqlite3.h>

#define RECORD_COUNTS {1000, 10000, 100000}
#define N_SCALES 3

typedef struct {
    int records;
    double time_ms;
    double cpu_user_ms;
    double cpu_sys_ms;
    long memory_kb;
    double ops_per_sec;
} BenchmarkResult;

typedef struct {
    BenchmarkResult import;
    BenchmarkResult export;
} ScaleResult;

static double now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec / 1e6;
}

static void get_metrics(double *cpu_user, double *cpu_sys, long *rss_kb) {
    struct rusage usage;
    getrusage(RUSAGE_SELF, &usage);
    *cpu_user = usage.ru_utime.tv_sec * 1000.0 + usage.ru_utime.tv_usec / 1000.0;
    *cpu_sys = usage.ru_stime.tv_sec * 1000.0 + usage.ru_stime.tv_usec / 1000.0;
    long rss = usage.ru_maxrss;
    #ifdef __APPLE__
    rss /= 1024;  // macOS reports bytes
    #endif
    *rss_kb = rss;
}

static int exec_sql(sqlite3 *db, const char *sql) {
    char *err = NULL;
    int rc = sqlite3_exec(db, sql, NULL, NULL, &err);
    if (rc != SQLITE_OK) {
        fprintf(stderr, "SQL error: %s\n", err);
        sqlite3_free(err);
    }
    return rc;
}

ScaleResult run_scale(int n) {
    ScaleResult r = {0};
    sqlite3 *db;
    char db_path[256];
    snprintf(db_path, sizeof(db_path), "/tmp/bench_c_%d.db", n);
    unlink(db_path);

    if (sqlite3_open(db_path, &db) != SQLITE_OK) {
        fprintf(stderr, "Cannot open db\n");
        exit(1);
    }
    exec_sql(db, "PRAGMA journal_mode=WAL;");
    exec_sql(db, "PRAGMA synchronous=OFF;");
    exec_sql(db, "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER, email TEXT, score REAL);");

    // --- IMPORT ---
    double cpu_u_before, cpu_s_before, cpu_u_after, cpu_s_after;
    long rss_before, rss_after;
    get_metrics(&cpu_u_before, &cpu_s_before, &rss_before);
    double t0 = now_ms();

    sqlite3_stmt *stmt;
    exec_sql(db, "BEGIN;");
    sqlite3_prepare_v2(db, "INSERT INTO users (id, name, age, email, score) VALUES (?, ?, ?, ?, ?);", -1, &stmt, NULL);
    for (int i = 0; i < n; i++) {
        char name[64], email[128];
        snprintf(name, sizeof(name), "User_%d", i);
        snprintf(email, sizeof(email), "user_%d@example.com", i);
        sqlite3_bind_int(stmt, 1, i + 1);
        sqlite3_bind_text(stmt, 2, name, -1, SQLITE_STATIC);
        sqlite3_bind_int(stmt, 3, 20 + (i % 50));
        sqlite3_bind_text(stmt, 4, email, -1, SQLITE_STATIC);
        sqlite3_bind_double(stmt, 5, (double)(i % 1000) + 0.99);
        sqlite3_step(stmt);
        sqlite3_reset(stmt);
    }
    sqlite3_finalize(stmt);
    exec_sql(db, "COMMIT;");

    double t1 = now_ms();
    get_metrics(&cpu_u_after, &cpu_s_after, &rss_after);

    r.import.records = n;
    r.import.time_ms = t1 - t0;
    r.import.cpu_user_ms = cpu_u_after - cpu_u_before;
    r.import.cpu_sys_ms = cpu_s_after - cpu_s_before;
    r.import.memory_kb = rss_after - rss_before;
    r.import.ops_per_sec = n / (r.import.time_ms / 1000.0);

    // --- EXPORT ---
    get_metrics(&cpu_u_before, &cpu_s_before, &rss_before);
    t0 = now_ms();

    sqlite3_prepare_v2(db, "SELECT * FROM users;", -1, &stmt, NULL);
    int count = 0;
    while (sqlite3_step(stmt) == SQLITE_ROW) {
        count++;
        (void)sqlite3_column_int(stmt, 0);
        (void)sqlite3_column_text(stmt, 1);
        (void)sqlite3_column_int(stmt, 2);
        (void)sqlite3_column_text(stmt, 3);
        (void)sqlite3_column_double(stmt, 4);
    }
    sqlite3_finalize(stmt);

    t1 = now_ms();
    get_metrics(&cpu_u_after, &cpu_s_after, &rss_after);

    r.export.records = n;
    r.export.time_ms = t1 - t0;
    r.export.cpu_user_ms = cpu_u_after - cpu_u_before;
    r.export.cpu_sys_ms = cpu_s_after - cpu_s_before;
    r.export.memory_kb = rss_after - rss_before;
    r.export.ops_per_sec = n / (r.export.time_ms / 1000.0);

    sqlite3_close(db);
    unlink(db_path);
    return r;
}

int main(void) {
    int scales[N_SCALES] = RECORD_COUNTS;
    ScaleResult results[N_SCALES];

    printf("C (libsqlite3) Benchmark\n");
    printf("=========================\n");

    for (int i = 0; i < N_SCALES; i++) {
        printf("Running %d records...\n", scales[i]);
        results[i] = run_scale(scales[i]);
    }

    // Output JSON
    FILE *f = fopen("results.json", "w");
    fprintf(f, "{\n");
    fprintf(f, "  \"language\": \"C\",\n");
    fprintf(f, "  \"library\": \"libsqlite3\",\n");
    fprintf(f, "  \"benchmarks\": {\n");
    for (int i = 0; i < N_SCALES; i++) {
        fprintf(f, "    \"n%d\": {\n", scales[i]);
        fprintf(f, "      \"import\": {\n");
        fprintf(f, "        \"records\": %d,\n", results[i].import.records);
        fprintf(f, "        \"time_ms\": %.2f,\n", results[i].import.time_ms);
        fprintf(f, "        \"cpu_user_ms\": %.2f,\n", results[i].import.cpu_user_ms);
        fprintf(f, "        \"cpu_sys_ms\": %.2f,\n", results[i].import.cpu_sys_ms);
        fprintf(f, "        \"memory_kb\": %ld,\n", results[i].import.memory_kb);
        fprintf(f, "        \"ops_per_sec\": %.0f\n", results[i].import.ops_per_sec);
        fprintf(f, "      },\n");
        fprintf(f, "      \"export\": {\n");
        fprintf(f, "        \"records\": %d,\n", results[i].export.records);
        fprintf(f, "        \"time_ms\": %.2f,\n", results[i].export.time_ms);
        fprintf(f, "        \"cpu_user_ms\": %.2f,\n", results[i].export.cpu_user_ms);
        fprintf(f, "        \"cpu_sys_ms\": %.2f,\n", results[i].export.cpu_sys_ms);
        fprintf(f, "        \"memory_kb\": %ld,\n", results[i].export.memory_kb);
        fprintf(f, "        \"ops_per_sec\": %.0f\n", results[i].export.ops_per_sec);
        fprintf(f, "      }\n");
        fprintf(f, "    }%s\n", i < N_SCALES - 1 ? "," : "");
    }
    fprintf(f, "  }\n");
    fprintf(f, "}\n");
    fclose(f);

    printf("\nResults written to results.json\n");
    for (int i = 0; i < N_SCALES; i++) {
        printf("\n%d records:\n", scales[i]);
        printf("  Import: %.2f ms (%.0f ops/sec), Memory: %ld KB\n",
               results[i].import.time_ms, results[i].import.ops_per_sec, results[i].import.memory_kb);
        printf("  Export: %.2f ms (%.0f ops/sec), Memory: %ld KB\n",
               results[i].export.time_ms, results[i].export.ops_per_sec, results[i].export.memory_kb);
    }

    return 0;
}
