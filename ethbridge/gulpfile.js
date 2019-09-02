const gulp = require("gulp");
const nearUtils = require("near-shell/gulp-utils");


function build_bindings(done){
  nearUtils.generateBindings("main.ts", "../out/main.near.ts", done);
}

function build_model(done){
  nearUtils.generateBindings("model.ts", "../out/model.near.ts", done);
}

function build_wasm(done){
  nearUtils.compile("../out/main.near.ts", "../out/main.wasm", done);
};

const build = gulp.series(build_model, build_bindings, build_wasm);


exports.default = build;
