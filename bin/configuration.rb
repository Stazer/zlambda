$matrix_dimension_size = 16
$matrix_element_size = 1
$matrix_size = $matrix_element_size * $matrix_dimension_size ** 2
$dataset_size = $matrix_size
$dataset_count = 10
$results_path = 'results'
$datasets_path = 'datasets'
$zlambda_cli_prefix = 'RUST_LOG="trace"'
$zlambda_cli_path = 'target/release/zlambda-cli'
$zlambda_matrix_process_path = 'target/release/zlambda-matrix-process'
$leader_server_address =
$server_addresses = [
  '192.168.8.2:8000',
  #'127.0.0.1:8001',
  #'127.0.0.1:8002',
]
$ebpf_socket_addresses = [
  '192.168.8.2:10200',
  #'192.168.8.3:10200',
  #'192.168.8.4:10200',
]
$parallel_execution_threads = 4

def available_datasets
  Dir.glob("#{$datasets_path}/*")
end

def available_result_names
  Dir.glob("#{$results_path}/*").map { |x| x.gsub("#{$results_path}/", '') }
end

def available_result_measurement_names(result_name)
  path = "#{$results_path}/#{result_name}/"

  Dir.glob("#{path}*").map { |x| x.gsub("#{path}", '') }
end
