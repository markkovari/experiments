import { afterAll, beforeAll } from 'vitest'
import { test, expect, describe } from '../setupTests'

describe("magic", () => {
    beforeAll(() => {
        console.log("beforeAll")
    })
    test("should workd", async ({ prisma }) => {
        try {
            const c = await prisma.user.count()
            console.log({ c })
            expect(c).toBe(0);
        } catch (error) {
            console.error(error)
            expect(error).toBeNull()
        }
    })
    afterAll(() => {
        console.log("afterAll")
    })
})