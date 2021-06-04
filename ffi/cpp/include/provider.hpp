// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

#ifndef ICU4X_PROVIDER_HPP
#define ICU4X_PROVIDER_HPP

#include <algorithm>
#include <optional>
#include <string_view>

#include "../../capi/include/provider.h"

namespace icu4x {
    class DataProvider {
    public:

        ~DataProvider() {
            if (this->inner._field1) {
                icu4x_data_provider_destroy(this->inner);
            }
        }
        DataProvider (const DataProvider&) = delete;
        DataProvider& operator= (const DataProvider&) = delete;
        DataProvider(DataProvider&& other) noexcept {
            this->inner = other.inner;
            other.inner = {0, 0};
        }
        DataProvider& operator=(DataProvider&& other) noexcept
        {
            std::swap(inner, other.inner);
            return *this;
        }
        static inline std::optional<DataProvider> FsDataProvider(const std::string_view& path) {
            ICU4XCreateDataProviderResult result = icu4x_fs_data_provider_create(path.data(), path.size());
            if (result.success) {
                return DataProvider(result.provider);
            } else {
                return {};
            }
        }
        inline ICU4XDataProvider AsFFI() const { return this->inner; }
    private:
        DataProvider(ICU4XDataProvider i) : inner(i) {}
        ICU4XDataProvider inner;
    };
};

#endif // ICU4X_PROVIDER_HPP