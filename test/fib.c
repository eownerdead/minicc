{
    int a; a = 1;
    int b; b = 1;
    int tmp;
    int i;
    for (i = 1; i <= 10; i = i + 1) {
        dbg(a);
        tmp = b;
        b = a + b;
        a = tmp;
    }
}
