
            match unsafe {  get_largest_window_x11(pid) } {
                Some(window_id) => {
                    println!("window id: {}", window_id);

                    // use it with ffmpeg
                    let _ffmpeg = Command::new("ffmpeg")
                        .args(&[
                            "-f", "x11grab",
                            "-framerate", "60",
                            "-window_id", &window_id,
                            "-i", ":0.0",
                            "-pix_fmt", "yuv420p",
                            "-vsync", "0",
                            "-fflags", "nobuffer",
                            "-c:v", "libx264",
                            "-preset", "ultrafast",
                            "-tune", "zerolatency",
                            "-bf", "0",
                            "-g", "15",
                            "-keyint_min", "15",
                            "-sc_threshold", "0",
                            "-b:v", "16m",
                            "-maxrate", "16m",
                            "-bufsize", "16m",
                            "-x264-params", "repeat-headers=1:rc-lookahead=0:no-scenecut=1:vbv-init=0",
                            "-f", "rtsp",
                            "-rtsp_transport", "tcp",
                            "rtsp://127.0.0.1:8554/1"
                        ])
                        .spawn()
                        .expect("Failed to start ffmpeg");
                }
                None => eprintln!("Could not find window for PID {}", pid),
            }
