
use sysinfo::System;
const SPLIT: usize = 2;

pub fn get_cpu_info(sys: &mut System, data: &mut Vec<f32>){
    data.clear();
    sys.refresh_cpu_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_usage();

    for cpu in sys.cpus() {
        // print!("{:.2}% ", cpu.cpu_usage());
        data.push(cpu.cpu_usage());
    }
    // println!();
}

pub fn draw_cpu(data : &Vec<f32>) -> String{
  const TOTAL: u8 = 4;

 let mut combined_output=String::new();
  let chunk_size = data.len() / SPLIT;
  for data_chunk in data.chunks(chunk_size){
  let mut output= String::new();
  for _ in data_chunk{
      output.push_str(" ┌──┐ ");
    }
    output.push_str("\n");

    for level in (0..TOTAL).rev(){
      for percentage in data_chunk{
          let filled_height=percentage/25.0; 
          if filled_height>level.into(){
            output.push_str(" │██│ ");
          } else{
           output.push_str(" │░░│ ")

          }
        }
    output.push_str("\n");
      }
    
    for _ in data_chunk{
     output.push_str(" └──┘ ");
    }
    output.push_str("\n");
    for (_i,p) in data_chunk.iter().enumerate(){
      // print!("{:^7}",i+1);
      output.push_str(&format!("{:^6}", format!("{:.1}%",p )));
    }
    output.push_str("\n");
    combined_output.push_str(&output);
  }
  combined_output
    
}






