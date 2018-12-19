import random

class Coupons:
    def __init__(self, n):
        self._n = n
        self._coupons = [False] * self._n

    def is_done(self):
        return all(self._coupons)

    def batch_coupons(self, k, rand):
        for _ in range(k):
            i = rand.randrange(self._n)
            self._coupons[i] = True
        return self.is_done()


def run():

    rand = random.Random()

    coupons = Coupons(2**11)
    num_iters = 0
    while not coupons.batch_coupons(1, rand):
        num_iters += 1

    print('num_iters = {}'.format(num_iters))

if __name__ == '__main__':
    run()
