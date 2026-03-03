<cfscript>
writeOutput("============================================================" & chr(10));
writeOutput("RustCFML Test Suite" & chr(10));
writeOutput("============================================================" & chr(10) & chr(10));

include "harness.cfm";

// --- Core Language ---
try { include "core/test_variables.cfm"; } catch (any e) { writeOutput("ERROR | core/test_variables.cfm | " & e.message & chr(10)); }
try { include "core/test_operators.cfm"; } catch (any e) { writeOutput("ERROR | core/test_operators.cfm | " & e.message & chr(10)); }
try { include "core/test_control_flow.cfm"; } catch (any e) { writeOutput("ERROR | core/test_control_flow.cfm | " & e.message & chr(10)); }
try { include "core/test_error_handling.cfm"; } catch (any e) { writeOutput("ERROR | core/test_error_handling.cfm | " & e.message & chr(10)); }
try { include "core/test_functions.cfm"; } catch (any e) { writeOutput("ERROR | core/test_functions.cfm | " & e.message & chr(10)); }
try { include "core/test_language_features.cfm"; } catch (any e) { writeOutput("ERROR | core/test_language_features.cfm | " & e.message & chr(10)); }
try { include "core/test_scopes.cfm"; } catch (any e) { writeOutput("ERROR | core/test_scopes.cfm | " & e.message & chr(10)); }
try { include "core/test_error_context.cfm"; } catch (any e) { writeOutput("ERROR | core/test_error_context.cfm | " & e.message & chr(10)); }

// --- Data Types ---
try { include "types/test_null.cfm"; } catch (any e) { writeOutput("ERROR | types/test_null.cfm | " & e.message & chr(10)); }
try { include "types/test_boolean.cfm"; } catch (any e) { writeOutput("ERROR | types/test_boolean.cfm | " & e.message & chr(10)); }
try { include "types/test_numeric.cfm"; } catch (any e) { writeOutput("ERROR | types/test_numeric.cfm | " & e.message & chr(10)); }
try { include "types/test_string.cfm"; } catch (any e) { writeOutput("ERROR | types/test_string.cfm | " & e.message & chr(10)); }
try { include "types/test_array.cfm"; } catch (any e) { writeOutput("ERROR | types/test_array.cfm | " & e.message & chr(10)); }
try { include "types/test_struct.cfm"; } catch (any e) { writeOutput("ERROR | types/test_struct.cfm | " & e.message & chr(10)); }
try { include "types/test_query.cfm"; } catch (any e) { writeOutput("ERROR | types/test_query.cfm | " & e.message & chr(10)); }
try { include "types/test_binary.cfm"; } catch (any e) { writeOutput("ERROR | types/test_binary.cfm | " & e.message & chr(10)); }

// --- Standard Library ---
try { include "stdlib/test_string_functions.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_string_functions.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_string_functions_regex.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_string_functions_regex.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_string_functions_encoding.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_string_functions_encoding.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_array_functions.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_array_functions.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_array_higher_order.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_array_higher_order.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_struct_functions.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_struct_functions.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_struct_higher_order.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_struct_higher_order.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_math_functions.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_math_functions.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_date_functions.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_date_functions.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_list_functions.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_list_functions.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_list_higher_order.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_list_higher_order.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_query_functions.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_query_functions.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_query_higher_order.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_query_higher_order.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_type_checking.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_type_checking.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_conversion.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_conversion.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_json.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_json.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_file_io.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_file_io.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_security.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_security.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_password_hashing.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_password_hashing.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_xml.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_xml.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_utility.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_utility.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_encoding_functions.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_encoding_functions.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_query_mutations.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_query_mutations.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_date_functions_extra.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_date_functions_extra.cfm | " & e.message & chr(10)); }
try { include "stdlib/test_locale_functions.cfm"; } catch (any e) { writeOutput("ERROR | stdlib/test_locale_functions.cfm | " & e.message & chr(10)); }

// --- Member Functions ---
try { include "members/test_string_members.cfm"; } catch (any e) { writeOutput("ERROR | members/test_string_members.cfm | " & e.message & chr(10)); }
try { include "members/test_array_members.cfm"; } catch (any e) { writeOutput("ERROR | members/test_array_members.cfm | " & e.message & chr(10)); }
try { include "members/test_struct_members.cfm"; } catch (any e) { writeOutput("ERROR | members/test_struct_members.cfm | " & e.message & chr(10)); }
try { include "members/test_number_members.cfm"; } catch (any e) { writeOutput("ERROR | members/test_number_members.cfm | " & e.message & chr(10)); }

// --- OOP ---
try { include "oop/test_components.cfm"; } catch (any e) { writeOutput("ERROR | oop/test_components.cfm | " & e.message & chr(10)); }
try { include "oop/test_inheritance.cfm"; } catch (any e) { writeOutput("ERROR | oop/test_inheritance.cfm | " & e.message & chr(10)); }
try { include "oop/test_interfaces.cfm"; } catch (any e) { writeOutput("ERROR | oop/test_interfaces.cfm | " & e.message & chr(10)); }
try { include "oop/test_metadata.cfm"; } catch (any e) { writeOutput("ERROR | oop/test_metadata.cfm | " & e.message & chr(10)); }

// --- Tags ---
try { include "tags/test_tags_basic.cfm"; } catch (any e) { writeOutput("ERROR | tags/test_tags_basic.cfm | " & e.message & chr(10)); }
try { include "tags/test_tags_control.cfm"; } catch (any e) { writeOutput("ERROR | tags/test_tags_control.cfm | " & e.message & chr(10)); }
try { include "tags/test_tags_include.cfm"; } catch (any e) { writeOutput("ERROR | tags/test_tags_include.cfm | " & e.message & chr(10)); }
try { include "tags/test_tags_savecontent.cfm"; } catch (any e) { writeOutput("ERROR | tags/test_tags_savecontent.cfm | " & e.message & chr(10)); }
try { include "tags/test_tags_param.cfm"; } catch (any e) { writeOutput("ERROR | tags/test_tags_param.cfm | " & e.message & chr(10)); }
try { include "tags/test_tags_misc.cfm"; } catch (any e) { writeOutput("ERROR | tags/test_tags_misc.cfm | " & e.message & chr(10)); }
try { include "tags/test_tags_customtag.cfm"; } catch (any e) { writeOutput("ERROR | tags/test_tags_customtag.cfm | " & e.message & chr(10)); }

printSummary();
</cfscript>
