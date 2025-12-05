package metrics

import "net/http"

// Wrapper around ResponseWriter to ensure the status code is saved for later usage
type responseWrapper struct {
	http.ResponseWriter
	statusCode int
}

// Save the written code locally after writing it to the actual ResponseWriter
func (res *responseWrapper) WriteHeader(statusCode int) {
	res.ResponseWriter.WriteHeader(statusCode)
	res.statusCode = statusCode
}
