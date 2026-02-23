<!--- Taffy API entry point - routing handled by Application.cfc --->
<cfscript>
// API request routing
if (structKeyExists(application, "_taffy")) {
	var pathInfo = "";
	if (structKeyExists(cgi, "path_info")) {
		pathInfo = cgi.path_info;
	}

	var method = "get";
	if (structKeyExists(cgi, "request_method")) {
		method = lCase(cgi.request_method);
	}

	var endpoints = application._taffy.endpoints;
	var matched = false;

	if (structKeyExists(endpoints, pathInfo)) {
		var endpoint = endpoints[pathInfo];
		var bean = endpoint.bean;

		if (structKeyExists(bean, method)) {
			try {
				var result = invoke(bean, method);
				writeOutput(serializeJSON(result));
			} catch(any e) {
				writeOutput(serializeJSON({"error": true, "message": e.message}));
			}
		} else {
			writeOutput(serializeJSON({"error": true, "message": "Method not allowed"}));
		}
		matched = true;
	}

	if (!matched) {
		writeOutput(serializeJSON({"error": true, "message": "No resource found for: " & pathInfo}));
	}
} else {
	writeOutput(serializeJSON({"error": true, "message": "Application not initialized"}));
}
</cfscript>