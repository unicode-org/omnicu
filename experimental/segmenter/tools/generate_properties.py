#!/usr/bin/env python3

# This file is part of ICU4X. For terms of use, please see the file
# called LICENSE at the top level of the ICU4X source tree
# (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

import os
import re
import subprocess

begin_plane2 = 0x20000

# Set default value for line break class to 'XX' (Unknown) for all the
# codepoints in Unicode Plane 0 and Plane 1
# https://www.unicode.org/reports/tr14/tr14-41.html#XX
lb_props = ["XX" for x in range(begin_plane2)]

# Set default value for East Asian Width Properties to 'Neutral' (Not East
# Asian) for all the codepoints in Unicode Plane 0 and Plane 1
# http://www.unicode.org/reports/tr11/#ED7
ea_props = ["N" for x in range(begin_plane2)]

rule = []
table = []

with open('EastAsianWidth.txt', 'r') as eaw_file:
    range_codepoint_pattern = r"([0-9A-F]{1,6})\.\.([0-9A-F]{1,6})\;([A-Za-z]{1,})"
    single_codepoint_pattern = r"([0-9A-F]{1,6})\;([A-Za-z]{1,})"

    for line in eaw_file.readlines():
        start = end = prop = None
        if m := re.match(range_codepoint_pattern, line):
            start = int(m[1], 16)
            end = int(m[2], 16)
            prop = m[3]
        elif m := re.match(single_codepoint_pattern, line):
            start = end = int(m[1], 16)
            prop = m[2]

        # We have a success match, and the codepoints are in plane 0 & 1. Store
        # their East Asian Width property.
        if prop and start < begin_plane2:
            for i in range(start, end + 1):
                ea_props[i] = prop

with open('LineBreak.txt', 'r') as lb_file:
    range_codepoint_pattern = r"([0-9A-F]{1,6})\.\.([0-9A-F]{1,6})\;([0-9A-Z]{2,})"
    single_codepoint_pattern = r"([0-9A-F]{1,6})\;([0-9A-Z]{2,})"

    for line in lb_file.readlines():
        start = end = prop = None
        if m := re.match(range_codepoint_pattern, line):
            start = int(m[1], 16)
            end = int(m[2], 16)
            prop = m[3]
        elif m := re.match(single_codepoint_pattern, line):
            start = end = int(m[1], 16)
            prop = m[2]

        # We have a success match, and the codepoints are in plane 0 & 1. Store
        # their line break class.
        if prop and start < begin_plane2:
            for i in range(start, end + 1):
                # for LB30
                # https://www.unicode.org/reports/tr14/tr14-45.html
                if prop == "OP":
                    if ea_props[i] in ("F", "W", "H"):
                        lb_props[i] = "OP_EA"
                    else:
                        lb_props[i] = "OP_OP30"
                elif prop == "CP" and ea_props[i] in ("F", "W", "H"):
                    # Note: This branch has no effect because the only two
                    # codepoints 0x0029 and 0x005D with CP line break class both
                    # have EA property "Na".
                    lb_props[i] = "CP_EA"
                else:
                    lb_props[i] = prop


prop_type = sorted([x for x in set(lb_props)])
prop_type.append("B2_SP")
prop_type.append("CL_CP_SP")
prop_type.append("HL_HY")
prop_type.append("LB25_HY")
prop_type.append("LB25_OP")
prop_type.append("LB25_NU_IS")
prop_type.append("LB25_NU_SY")
prop_type.append("LB25_NU_CL")
prop_type.append("LB25_NU_CP")
prop_type.append("OP_SP")
prop_type.append("QU_SP")
prop_type.append("RI_RI")
prop_type.append("EOT")
for i in prop_type:
    back_i = i
    for j in prop_type:
        i = back_i

        # AI
        if i == "AI":
            i = "AL"
        if j == "AI":
            j = "AL"

        # break-strict -> NS, others -> ID
        if i == "CJ":
            i = "NS"
        if j == "CJ":
            j = "NS"

        # SA
        if i == "SA":
            i = "AL"
        if j == "SA":
            j = "AL"

        # LB1
        if i == "XX":
            i = "AL"
        if j == "XX":
            j = "AL"

        # LB2
        # LB3
        if j == "EOT":
            if i in ("LB25_OP", "LB25_HY"):
                rule.append("f") # failed. Previous is break.
                continue
            rule.append("!")
            continue

        # LB4
        if i == "BK":
            rule.append("!")
            continue

        # LB5
        if i == "CR" and j == "LF":
            rule.append("x")
            continue
        if i in ("CR", "LF", "NL"):
            rule.append("!")
            continue

        # LB6
        if j in ("BK", "CR", "LF", "NL"):
            rule.append("x")
            continue

        # LB7
        if j == "SP":
            # (LB8)
            if i == "ZW":
                rule.append(i)
                continue
            # (LB14)
            if i in ("OP_OP30", "OP_EA", "OP_SP"):
                rule.append("OP_SP")
                continue
            # (LB15)
            if i in ("QU", "QU_SP"):
                rule.append("QU_SP")
                continue
            # (LB16)
            if i in ("CL", "CP", "CP_EA", "CL_CP_SP"):
                rule.append("CL_CP_SP")
                continue
            # (LB17)
            if i in ("B2", "B2_SP"):
                rule.append("B2_SP")
                continue

        if j in ("SP", "ZW"):
            rule.append("x")
            continue

        # LB8
        if i in ("ZW"):
            rule.append("/")
            continue

        # LB8a
        if i == "ZWJ":
            rule.append("x")
            continue

        # LB9
        if (i not in ("BK", "CR", "LF", "NL", "SP", "ZW", "B2_SP", "QU_SP",
                     "CL_CP_SP")) and j in ("CM", "ZWJ"):
            rule.append(i)
            continue

        # LB10
        if i == "CM":
            i = "AL"
        if j == "CM":
            j = "AL"
        if j == "ZWJ":
            j = "AL"

        # LB11
        if i == "WJ":
            rule.append("x")
            continue
        if j == "WJ":
            rule.append("x")
            continue

        # LB12
        if i == "GL":
            rule.append("x")
            continue

        # LB12a
        if j == "GL":
            if i in ("B2_SP", "CL_CP_SP", "QU_SP", "SP", "BA", "HY"):
                rule.append("/")
                continue
            rule.append("x")
            continue

        # LB13
        if i in ("NU", "LB25_NU_SY", "LB25_NU_IS") and j in ("IS", "SY", "CL", "CP", "CP_EA"):
            # LB25 rule.
            pass
        elif j in ("CL", "CP", "CP_EA", "EX", "IS", "SY"):
            rule.append("x")
            continue

        # LB14
        if i in ("OP_OP30", "OP_EA", "OP_SP"):
            rule.append("x")
            continue

        # LB15
        if i in ("QU", "QU_SP") and j in ("OP_OP30", "OP_EA"):
            rule.append("x")
            continue
        if i == "QU_SP":
            i = "SP"

        # LB 16
        if i in ("CL", "CP", "CP_EA", "CL_CP_SP", "LB25_NU_CL", "LB25_NU_CP") and j == "NS":
            rule.append("x")
            continue
        if i == "CL_CP_SP":
            i = "SP"

        # LB17
        if i in ("B2", "B2_SP") and j == "B2":
            rule.append("x")
            continue
        if i == "B2_SP":
            i = "SP"

        # LB18
        if i == "SP":
            rule.append("/")
            continue

        # LB19
        if i == "QU":
            rule.append("x")
            continue
        if j == "QU":
            rule.append("x")
            continue

        # LB20
        if i == "CB":
            rule.append("/")
            continue
        if j == "CB":
            rule.append("/")
            continue

        # LB21
        # (LB21a)
        if i == "HL" and j in ("HY", "BA"):
            rule.append("HL_HY")
            continue
        if j in ("BA", "HY", "NS"):
            rule.append("x")
            continue
        if i == "BB":
            rule.append("x")
            continue

        # LB21a
        if i == "HL_HY":
            rule.append("x")
            continue

        # LB21b
        if i == "SY" and j == "HL":
            rule.append("x")
            continue

        # LB22
        if j == "IN":
            rule.append("x")
            continue

        # LB23
        if i in ("AL", "HL") and j == "NU":
            rule.append("x")
            continue
        if i == "NU" and j in ("AL", "HL"):
            rule.append("x")
            continue

        # LB23a
        if i == "PR" and j in ("ID", "EB", "EM"):
            rule.append("x")
            continue
        if i in ("ID", "EB", "EM") and j == "PO":
            rule.append("x")
            continue

        # LB24
        if i in ("PR", "PO") and j in ("AL", "HL"):
            rule.append("x")
            continue
        if i in ("AL", "HL") and j in ("PR", "PO"):
            rule.append("x")
            continue

        # LB25
        # (PR|PO) ? (OP|HY) ? NU (NU|SY|IS) * (CL|CP) ? (PR|PO) ?
        if i in ("PR", "PO") and j in ("OP_OP30", "OP_EA"):
            rule.append("LB25_OP")
            continue
        if i in ("PR", "PO") and j in ("HY"):
            rule.append("LB25_HY")
            continue
        if i in ("PR", "PO", "OP_OP30", "OP_EA", "HY", "LB25_OP", "LB25_HY") and j in ("NU"):
            rule.append("x")
            continue
        if i in ("NU", "LB25_NU_IS", "LB25_NU_SY") and j == "NU":
            rule.append("NU")
            continue
        if i in ("NU", "LB25_NU_IS", "LB25_NU_SY") and j == "SY":
            rule.append("LB25_NU_SY")
            continue
        if i in ("NU", "LB25_NU_IS", "LB25_NU_SY") and j == "IS":
            rule.append("LB25_NU_IS")
            continue
        if i in ("NU", "LB25_NU_IS", "LB25_NU_SY") and j in ("CL"):
            rule.append("LB25_NU_CL")
            continue
        if i in ("NU", "LB25_NU_IS", "LB25_NU_SY") and j in ("CP"):
            rule.append("LB25_NU_CP")
            continue
        if i in ("NU", "LB25_NU_IS", "LB25_NU_SY", "LB25_NU_CL", "LB25_NU_CP") and j in ("PR", "PO"):
            rule.append("x")
            continue

        # Restore
        if i in ("LB25_OP", "LB25_HY"):
            rule.append("f") # failed. Previous is break
            continue

        if i == "LB25_NU_IS":
            i = "IS"
        elif i == "LB25_NU_SY":
            i = "SY"
        elif i == "LB25_NU_CL":
            i = "CL"
        elif i == "LB25_NU_CP":
            i = "CP"

        # LB26
        if i == "JL" and j in ("JL", "JV", "H2", "H3"):
            rule.append("x")
            continue
        if i in ("JV", "H2") and j in ("JV", "JT"):
            rule.append("x")
            continue
        if i in ("JT", "H3") and j == "JT":
            rule.append("x")
            continue

        # LB27
        if i in ("JL", "JV", "JT", "H2", "H3") and j == "IN":
            rule.append("x")
            continue
        if i in ("JL", "JV", "JT", "H2", "H3") and j == "PO":
            rule.append("x")
            continue
        if i == "PR" and j in ("JL", "JV", "JT", "H2", "H3"):
            rule.append("x")
            continue

        # LB28
        if i in ("AL", "HL") and j in ("AL", "HL"):
            rule.append("x")
            continue

        # LB29
        if i in ("IS") and j in ("AL", "HL"):
            rule.append("x")
            continue

        # XXX LB30
        if i in ("AL", "HL", "NU") and j == "OP_OP30":
            rule.append("x")
            continue
        if i == "CP" and j in ("AL", "HL", "NU"):
            rule.append("x")
            continue

        # LB30a
        if i == "RI" and j == "RI":
            rule.append("RI_RI")
            continue
        if i == "RI_RI" and j == "RI":
            rule.append("/")
            continue
        if i == "RI_RI":
            i = "RI"

        # LB30b
        if i == "EB" and j == "EM":
            rule.append("x")
            continue

        rule.append("/")

header = """// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

// This file is generated by generate_properties.py. DO NOT EDIT MANUALLY!

"""

with open('lb_define.rs', 'w') as prop_file:
    prop_file.write(header)

    prop_len = len(prop_type)
    count = 1
    for i in prop_type:
        prop_file.write("pub const %s: u8 = %s;\n" % (i, str(count)))
        count = count + 1

    prop_file.write("pub const PROP_COUNT: usize = %d;\n" % (count - 1))
    prop_file.write("\n")

    prop_file.write("#[allow(dead_code)]\n")
    prop_file.write("pub const BREAK_RULE: i8 = -128;\n")
    prop_file.write("pub const PREVIOUS_BREAK_RULE: i8 = -2;\n")
    prop_file.write("pub const KEEP_RULE: i8 = -1;\n")
    prop_file.write("\n")

# For Line break property
with open('properties_defines.rs', 'w') as properties_file:
    properties_file.write(header)
    properties_file.write("use crate::lb_define::*;\n\n")

    for a in range(128):
        first_value = lb_props[a * 1024]
        generate_table = False
        for i in range(1024):
            if lb_props[a * 1024 + i] != first_value:
                generate_table = True
                break

        if not generate_table:
            table.append("UAX14_PROPERTIES_%s" % first_value)
            continue

        properties_file.write("pub const UAX14_PROPERTIES_%s: [u8; 1024] = [\n" % str(a))
        for i in range(int(1024)):
            properties_file.write(" %s," % lb_props[a * 1024 + i])
        properties_file.write("];\n\n")

        table.append("UAX14_PROPERTIES_%s" % str(a))

# For Line break property
with open('properties_other.rs', 'w') as properties_file:
    properties_file.write(header)
    properties_file.write("use crate::lb_define::*;\n\n")

    for t in ["ID", "SG", "XX"]:
        properties_file.write("pub const UAX14_PROPERTIES_%s: [u8; 1024] = [\n" % t)
        for i in range(int(1024 / 16)):
            properties_file.write(" ")
            for j in range(16):
                properties_file.write(" %s," % t)
            properties_file.write("\n");
        properties_file.write("];\n\n");

# For Line break property
with open('property_table.rs', 'w') as table_file:
    table_file.write(header)
    table_file.write("use crate::properties_defines::*;\n");
    table_file.write("use crate::properties_other::*;\n\n");

    table_file.write("pub const UAX14_PROPERTY_TABLE: [&[u8; 1024]; 128] = [\n")
    for i in table:
        table_file.write("  &%s,\n" % i)
    table_file.write("];\n")

with open('rule_table.rs', 'w') as table_file:
    table_file.write(header)
    table_file.write("use crate::lb_define::*;\n\n")
    table_file.write("pub const UAX14_RULE_TABLE: [i8; %d] = [\n" % len(rule))
    count = 0
    line = 0
    table_file.write("// %s\n" % prop_type[line])
    for i in rule:
        value = 0
        # handing state machine
        if i == "x":
            value = -1
        elif i == "f":
            value = -2
        elif i == "/":
            value = -128
        elif i == "!":
            value = -128
        else:
            value = "%s as i8" % i
        table_file.write(" %s," % str(value))
        count = count + 1
        if count >= len(prop_type):
            count = 0
            line = line + 1
            try:
                table_file.write("\n// %s\n" % prop_type[line])
            except:
                pass
    table_file.write("];\n")


# Format and copy generated rs files to src directory.

# properties_defines.rs, properties_other.rs and property_table.rs are Line Break property table map for UAX14
subprocess.call(["rustfmt", "properties_defines.rs"])
os.rename("properties_defines.rs", "../src/properties_defines.rs")
subprocess.call(["rustfmt", "properties_other.rs"])
os.rename("properties_other.rs", "../src/properties_other.rs")
subprocess.call(["rustfmt", "property_table.rs"])
os.rename("property_table.rs", "../src/property_table.rs")
# lb_defines.rs is custom state machine defines.
subprocess.call(["rustfmt", "lb_define.rs"])
os.rename("lb_define.rs", "../src/lb_define.rs")
# rule_table.rs is state machine table by UAX14
subprocess.call(["rustfmt", "rule_table.rs"])
os.rename("rule_table.rs", "../src/rule_table.rs")
