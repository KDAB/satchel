#include <algorithm>
#include <vector>

extern "C" int some_tests_main();
extern "C" int other_tests_main();

int main()
{
    std::vector<int> results;

    results.push_back(some_tests_main());
    results.push_back(other_tests_main());

    return *std::max_element(results.cbegin(), results.cend());
}
