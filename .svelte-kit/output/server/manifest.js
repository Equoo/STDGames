export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set([]),
	mimeTypes: {},
	_: {
		client: {start:"_app/immutable/entry/start.joqyo5xh.js",app:"_app/immutable/entry/app.CCMv3HDa.js",imports:["_app/immutable/entry/start.joqyo5xh.js","_app/immutable/chunks/DPljhWEC.js","_app/immutable/chunks/wuoRkyL-.js","_app/immutable/chunks/D1-duvBn.js","_app/immutable/chunks/Cas-dNfL.js","_app/immutable/entry/app.CCMv3HDa.js","_app/immutable/chunks/PPVm8Dsz.js","_app/immutable/chunks/wuoRkyL-.js","_app/immutable/chunks/DHYVup6P.js","_app/immutable/chunks/CwhMuEnY.js","_app/immutable/chunks/Cas-dNfL.js","_app/immutable/chunks/CmHTnYj5.js","_app/immutable/chunks/Xmai3W7a.js","_app/immutable/chunks/D1-duvBn.js","_app/immutable/chunks/BTakXyPK.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js')),
			__memo(() => import('./nodes/2.js')),
			__memo(() => import('./nodes/3.js'))
		],
		remotes: {
			
		},
		routes: [
			{
				id: "/",
				pattern: /^\/$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 2 },
				endpoint: null
			},
			{
				id: "/splashscreen",
				pattern: /^\/splashscreen\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 3 },
				endpoint: null
			}
		],
		prerendered_routes: new Set([]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
