

fn add_launcher_desktop_icon()
{

    // let mut file = File::create("foo.txt")?;
    // file.write_all(b"Hello, world!")?;

    // let mut file = File::open("foo.txt")?;
    // let mut contents = String::new();
    // file.read_to_string(&mut contents)?;
    // assert_eq!(contents, "Hello, world!"); 

    // let file = File::open("foo.txt")?;
    // let mut buf_reader = BufReader::new(file);
    // let mut contents = String::new();
    // buf_reader.read_to_string(&mut contents)?;
    // assert_eq!(contents, "Hello, world!");


    // let mut file File::

    let desktop = Path::new(std::env("HOME").unwrap());
    desktop.join(".local/share/.application/STDGames.desktop");

    std::fs::soft_link("/sgoinfre/42GamingNight/.STDGames/Ressources/STDGames.desktop", desktop).unwrap();


}

