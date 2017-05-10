Sam Sartor / Dr. Occludo / ssartor@mymail.mines.edu
Assignment 4 / Shadows!

Description
===========

	This program loads some models out of obj files and renders them with
	phong shading and shadow maps.
	
Usage
=====

	The camera is an arc-ball. Click+Drag to rotate. Scroll to zoom.

	"esc" exits the program. "f" toggles the floor.

	Make sure to run the project in the same directory as
	"teapot.obj", "floor.obj", and "shaders/".

Details
=======

	I implemented this project using a language I am particularly fond of
	called "Rust". I like it for all sorts of reasons and I could spend months
	listing them off, but you don't want to read that.

	There are generally 3 options for doing OpenGL in Rust:

		1. gl-generator - Generates C-level OpenGL bindings directly from the
		OpenGL headers. Hard to use because of raw pointers, null-terminated
		strings, state-machine-ness, and other non-rusty sorts of things. Some
		sort of wrapper libraries/functions are needed to make it reasonably
		usable.

		2. Glium - A more rusty way of using OpenGL, not finished so some
		things (like FBOs) require nasty hacks. No longer supported for a few
		reasons (one of which is that gfx is better).

		3. gfx - Totally awesome! Provides a high-level, pipeline-oriented
		superset of OpenGL, Direct3D, Metal (WIP), Vulcan (WIP), and so on.
		Very powerful, easy to multi-thread, but does still require a thorough
		knowledge of OpenGL pipeline, buffer allocation/layout, and shaders
		(still very very low-level, we aren't talking about Unreal Engine
		here).

	For my previous projects I have used Glium because it and gl-generator
	were the only options I knew of. Recently I did some more research and
	came across gfx, which I used for this project instead. That did require
	re-writing (or at least heavily adapting/modifying) my existing code from
	projects 1-3, but given the poor support Glium has for custom FBOs, I
	think it was worth it.

	As I said above, gfx can use DirectX 11/DirectX 12/Metal/Vulcan as
	backends. However, that does require writing separate shaders for each
	platform and insuring consistent buffer layout between them. Given that
	this class is focused on OpenGL, this program just uses OpenGL always.

	The render passes are in "src/app.rs::App::render()".
	"src/app.rs::App::new()" sets up all the pipeline stuff which is defined
	using a cool gfx macro in "src/define.rs".

Building
========

	If you have Rust installed (https://www.rustup.rs/), execute "cargo run"
	in the project's main directory. Unlike projects 1-3, nightly is NOT
	required. Nightly is generally better though, so just use it if you have
	it installed. I have also included linux and windows executables.

Time: ~9h
Fun: 9, I was a bit panicked when starting out (mainly because of the gfx re-
	write), but I ended up enjoying it
