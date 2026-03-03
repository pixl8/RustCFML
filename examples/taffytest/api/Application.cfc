<cfcomponent>
	<cfscript>

		this.name = "TaffyTestAPI";
		this.mappings = {
			"/taffy": expandPath("../../../../Taffy"),
			"/resources": expandPath("./resources")
		};

		function onApplicationStart(){
			var factory = createObject("component", expandPath("../../../../Taffy/core/factory.cfc"));
			factory.init();

			var resourcePath = expandPath("./resources");
			factory.loadBeansFromPath(resourcePath, "resources", resourcePath, true, { status: { skippedResources: [] }, beanList: "" });

			var beanList = factory.getBeanList();

			// Build endpoints map from loaded beans
			var endpoints = {};
			var uriOrder = [];
			var beanNames = listToArray(beanList);
			var i = 0;
			for (i = 1; i <= arrayLen(beanNames); i = i + 1) {
				var beanName = beanNames[i];
				var bean = factory.beans[beanName];
				if (structKeyExists(bean, "__metadata")) {
					var meta = bean.__metadata;
					if (structKeyExists(meta, "taffy_uri")) {
						var uri = meta.taffy_uri;
						endpoints[uri] = {
							beanName: beanName,
							bean: bean
						};
						arrayAppend(uriOrder, uri);
					}
				}
			}

			application._taffy = {
				factory: factory,
				beanList: beanList,
				endpoints: endpoints,
				URIMatchOrder: uriOrder
			};

			return true;
		}

		function onRequestStart(TARGETPATH){
			return true;
		}


	</cfscript>
</cfcomponent>
