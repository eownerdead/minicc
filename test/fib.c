int fib(int n)
{
    if (n == 0) {
        return 0;
    } else if (n == 1) {
        return 1;
    } else {
        return fib(n - 1) + fib(n - 2);
    }
}

int main()
{
    int i;
    for (i = 1; i <= 25; i = i + 1) {
        dbg(fib(i));
    }
}
