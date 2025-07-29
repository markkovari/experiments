import { test, expect, describe } from '../testBed'


describe("magic", () => {
    test("should workd", async ({ prisma }) => {
        const c = await prisma.user.count()
        console.log({ c })
        expect(c).toBe(0);
    })
})