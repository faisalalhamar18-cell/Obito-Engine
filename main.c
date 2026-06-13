#include <stdio.h>

// الإعلان عن الدالة القادمة من مكتبة Rust (Obito_Engine)
extern void create_obito_window(const char* title, unsigned int width, unsigned int height);

int main() {
    printf("Starting Obito Engine Management System...\n");
    
    // تشغيل النافذة بأبعاد 1280x720 وعنوان المحرك
    create_obito_window("Obito Engine - AAA Core", 1280, 720);
    
    return 0;
}