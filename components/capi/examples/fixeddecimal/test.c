// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

#include "../../include/decimal.h"
#include <string.h>
#include <stdio.h>

const char* path = "../../../../resources/testdata/data/json/";
int main() {
    ICU4XLocale* locale = icu4x_locale_create("bn", 2);
    ICU4XCreateDataProviderResult result = icu4x_fs_data_provider_create(path, strlen(path));
    if (!result.success) {
        printf("Failed to create FsDataProvider\n");
        return 1;
    }
    ICU4XDataProvider provider = result.provider;
    ICU4XCreateFixedDecimalResult decimal_result = icu4x_fixed_decimal_create(1000007, 0);
    if (!decimal_result.success)  {
        printf("Failed to create FixedDecimal\n");
        return 1;
    }
    ICU4XFixedDecimal* decimal = decimal_result.decimal;
    ICU4XFixedDecimalFormatOptions opts = {ICU4XGroupingStrategy_Auto, ICU4XSignDisplay_Auto};

    ICU4XCreateFixedDecimalFormatResult fdf_result = icu4x_fixed_decimal_format_create(locale, &provider, opts);
    if (!fdf_result.success)  {
        printf("Failed to create FixedDecimalFormat\n");
        return 1;
    }
    ICU4XFixedDecimalFormat* fdf = fdf_result.fdf;
    char output[40];

    ICU4XCustomWriteable write = icu4x_simple_writeable(output, 40);

    bool success = icu4x_fixed_decimal_format_format(fdf, decimal, &write);
    if (!success) {
        printf("Failed to write result of FixedDecimalFormat::format to string.\n");
        return 1;
    }
    printf("Output is %s\n", output);

    char* expected = "১০,০০,০০৭";
    if (strcmp(output, expected) != 0) {
        printf("Output does not match expected output!\n");
        return 1;
    }

    return 0;
}
