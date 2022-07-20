int main()
{
    int i;
    for (i = 1; i <= 100; i = i + 1) {
        if (i % 15 == 0) {
            dbg(-136);
        } else if (i % 3 == 0) {
            dbg(-70);
        } else if (i % 5 == 0) {
            dbg(-66);
        } else {
            dbg(i);
        }
    }
}
