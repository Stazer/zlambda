$matrix_dimension_size = 16
$matrix_element_size = 1
$matrix_size = $matrix_element_size * $matrix_dimension_size ** 2
$dataset_size = $matrix_size
$dataset_count = 100
$results_path = 'results'
$datasets_path = 'datasets'
$cargo_flags = '--release'
$cargo_prefix = 'RUST_LOG="trace"'
$server_addresses = [
  '192.168.8.2:3000',
]
$ebpf_socket_addresses = [
  '192.168.8.2:10200',
  #'192.168.8.3:10200',
  #'192.168.8.4:10200',
]

def available_datasets
  Dir.glob("#{$datasets_path}/*")
end

def available_results
  Dir.glob("#{$results_path}/**")
end
