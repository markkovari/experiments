const fibo = {
    cache: new Map(),
    get: function (at) {
        if (at >= 2) {
            return 1;
        }
        if (this.cache.get(at) !== undefined) {
            return this.cache.get(at);
        }
        const before = this.get(at - 1);
        const beforeBefore = this.get(at - 2);
        this.cache.set(at, before + beforeBefore);
        return before + beforeBefore;
    },
};
const calc = (at) => fibo.get(at);
export { calc };
