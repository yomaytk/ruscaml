#include <stdio.h>
#include <stdlib.h>

extern int _toplevel();

void *mymalloc(int n) {
  return malloc(n * sizeof(int));
}

int main() {
    return _toplevel();
}