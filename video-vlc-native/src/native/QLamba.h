#ifndef POPCORNPLAYER_QLAMBA_H
#define POPCORNPLAYER_QLAMBA_H

#include <tuple>

using namespace std;

namespace qlambda {

/**
 * Defines an abstract QLambda which is runnable within the QApplication.
 * This class shouldn't be used directly, use QLambda instead.
 *
 * @see QLambda
 */
class AbstractQLambda {
public:
    virtual ~AbstractQLambda() = default;

    virtual void run() = 0;
};


template <class Lambda, class... Args>
class QLambda : public AbstractQLambda {
public:
    QLambda(Lambda lambda, Args... args)
        : AbstractQLambda()
        , _lambda(lambda)
        , _args(std::make_tuple(args...)){};

    void run() override
    {
        internalRun(std::make_index_sequence<sizeof...(Args)>());
    }

private:
    Lambda _lambda;
    std::tuple<Args...> _args;

    template <std::size_t... I>
    void internalRun(std::index_sequence<I...>)
    {
        _lambda(std::get<I>(_args)...);
    }
};

}

#endif //POPCORNPLAYER_QLAMBA_H
