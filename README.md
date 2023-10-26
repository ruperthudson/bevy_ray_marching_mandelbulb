# Bevy Mandelbulb Ray Marching Project

This project showcases a sophisticated ray marcher built using the Bevy game engine, demonstrating the capabilities of distance-aided ray marching techniques. It utilizes the power of shaders and GPU computing to visualize complex fractal structures, specifically the Mandelbulb fractal.

## Features:

- **Mandelbulb Visualization**: The shader calculates the Mandelbulb fractal, a three-dimensional fractal that exhibits intricate and captivating patterns. The formula and rendering details are encapsulated in the shader code.

- **Adaptive Screen Space**: The ray marcher utilizes a custom screen space quad to display the fractal. This ensures that the visualization adjusts according to the screen's aspect ratio, particularly during window resizing.

- **Dynamic Lighting**: The shader incorporates multiple light sources, including directional and downward-facing lights, to illuminate the fractal. This is combined with ambient lighting, specular highlights, and ambient occlusion techniques to create a visually appealing result.

- **Interactive Camera**: Users can navigate the 3D space using the keyboard and mouse. This allows for exploration and closer inspection of the Mandelbulb's fascinating structures. The camera movement is smooth and intuitive, allowing for rotation, panning, and zooming.

- **UI Integration**: The application integrates with the Bevy's Egui plugin, providing a user interface for real-time parameter adjustments and other controls.

## Preview:

The current visualization of the Mandelbulb fractal is as shown below:
![](https://github.com/Lowband21/bevy_ray_marching_mandelbulb/blob/master/assets/output/Mandelbulb.png)

## Usage:

To get started with the project, ensure you have the Bevy game engine set up and the necessary dependencies installed. Clone the repository, navigate to the project directory, and run the application. Use the mouse and keyboard controls to interact with the visualization and explore the Mandelbulb fractal in its full glory.

## Contributions:

This project is open to contributions. If you have suggestions, optimizations, or want to add new features, feel free to fork the repository and submit a pull request.

Enjoy exploring the depths of fractal visualization with the Bevy Mandelbulb Ray Marching Project!

## Credits:

Thank you so much to lukeduball for his bevy_ray_marching repository, from which this project was originally forked.
