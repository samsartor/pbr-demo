Sam Sartor / ssartor@mines.edu
Daichi Jameson / djameson@mines.edu>
Final Project / Physically Based Rendering

Description
===========

	This program demonstrates BPR shading on a variety of textured models.
	
Usage
=====

	The program takes several command line arguments. You can use --help to
	list them off, but generally you only need to worry about the -o argument.
	It is followed by a list of directories. Each directory should contain:
		model.obj (including texture and normal coordinates)
		normal.png
		albedo.png
		metalness.png
		roughness.png

	A set of example directories is found in "objects/".

	In general, the command you want is:
		[program executable] -o objects/buddha_wood objects/cerberus objects/painted_metal objects/rusty_car objects/teapot_wood

	The camera is an arc-ball. Click+Drag to rotate. Scroll to zoom.

	"esc" exits the program. "m" cycles through the available objects (from
	the directory list). "c" toggles between default light colors and
	randomized light colors. Up/Down adjusts exposure. Right/Left adjusts gamma.

Details
=======

	We implemented this project using a language Sam is particularly fond of
	called "Rust". He likes it for all sorts of reasons and he could spend
	months listing them off, but you don't want to read that.

	High-level, robust, and multi-threadable access to OpenGL is provided
	through a library called gfx. Gfx can also use DirectX 11, DirectX 12,
	Metal, and Vulcan as backends. However, that does require writing separate
	shaders for each platform and insuring consistent buffer layout between
	them. Given that this class is focused on OpenGL, this program just uses
	OpenGL always.

	The render function is src/app.rs::App::render(). src/app.rs::App::new()
	sets up all the pipeline stuff which is defined using a cool gfx macro in
	src/define.rs. The render passes are:
 	
 	 	- Render scene to gbuffer
 	 	For each light:
 	 		- Render scene to shadowbuffer
 	 		- Do deferred pass for single light (additive blending into luminance buffer)
 	 	- Do post processing (convert HDR luminance buffer to LDR output)

	Further details are in the paper.

Building
========

	If you have Rust installed (https://www.rustup.rs/), execute "cargo run
	--release -- -o [objects]" in the project's main directory. Nightly is NOT
	required, but it is generally better. I have also included linux and
	windows executables.

Time: ~12h (not including copy-and-pasted code from previous projects)
Fun: ... (PBR is AWESOME!)
