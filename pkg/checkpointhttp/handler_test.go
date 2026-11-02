package checkpointhttp

import (
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestIsAlignedCheckpoint(t *testing.T) {
	if !IsAlignedCheckpoint(63) || !IsAlignedCheckpoint(0x03dca33f) {
		t.Fatalf("expected aligned checkpoints to be valid")
	}
	if IsAlignedCheckpoint(64) {
		t.Fatalf("expected 64 to be rejected")
	}
}

func TestComputeCheckpointETag(t *testing.T) {
	etag := ComputeCheckpointETag(64791295, "03dca33f89abcdef")
	expected := `"ckpt-64791295-03dca33f89abcdef"`
	if etag != expected {
		t.Fatalf("ComputeCheckpointETag = %s, want %s", etag, expected)
	}

	// Truncates long prefixes to 16 hex characters
	etagLong := ComputeCheckpointETag(63, "abcdef0123456789extraextra")
	expectedLong := `"ckpt-63-abcdef0123456789"`
	if etagLong != expectedLong {
		t.Fatalf("ComputeCheckpointETag = %s, want %s", etagLong, expectedLong)
	}
}

func TestServeVerifiedCheckpoint_TableDrivenConditionalCaching(t *testing.T) {
	body := []byte(`{"checkpoint_seq":63,"verified":true}`)
	seq := uint32(63)
	prefix := "a1b2c3d4"
	expectedETag := fmt.Sprintf(`"ckpt-%d-%s"`, seq, prefix)

	tests := []struct {
		name           string
		ifNoneMatch    string
		wantStatus     int
		wantBodyEmpty  bool
		wantETagHeader string
	}{
		{
			name:           "no If-None-Match header -> 200 OK with body",
			ifNoneMatch:    "",
			wantStatus:     http.StatusOK,
			wantBodyEmpty:  false,
			wantETagHeader: expectedETag,
		},
		{
			name:           "matching If-None-Match header -> 304 Not Modified empty body",
			ifNoneMatch:    expectedETag,
			wantStatus:     http.StatusNotModified,
			wantBodyEmpty:  true,
			wantETagHeader: expectedETag,
		},
		{
			name:           "wildcard If-None-Match header -> 304 Not Modified empty body",
			ifNoneMatch:    "*",
			wantStatus:     http.StatusNotModified,
			wantBodyEmpty:  true,
			wantETagHeader: expectedETag,
		},
		{
			name:           "non-matching If-None-Match header -> 200 OK with body",
			ifNoneMatch:    `"ckpt-62-different"`,
			wantStatus:     http.StatusOK,
			wantBodyEmpty:  false,
			wantETagHeader: expectedETag,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, "/v1/checkpoints/63", nil)
			if tt.ifNoneMatch != "" {
				req.Header.Set("If-None-Match", tt.ifNoneMatch)
			}
			rec := httptest.NewRecorder()

			ServeVerifiedCheckpoint(rec, req, seq, prefix, body)

			if rec.Code != tt.wantStatus {
				t.Fatalf("status = %d, want %d", rec.Code, tt.wantStatus)
			}

			// Verify RFC 9110 header preservation on 304 and 200
			if got := rec.Header().Get("ETag"); got != tt.wantETagHeader {
				t.Fatalf("ETag header = %s, want %s", got, tt.wantETagHeader)
			}
			if got := rec.Header().Get("Cache-Control"); got != ImmutableCacheControl {
				t.Fatalf("Cache-Control header = %s, want %s", got, ImmutableCacheControl)
			}

			if tt.wantBodyEmpty && rec.Body.Len() != 0 {
				t.Fatalf("expected empty body on 304, got %d bytes: %q", rec.Body.Len(), rec.Body.String())
			}
			if !tt.wantBodyEmpty && rec.Body.String() != string(body) {
				t.Fatalf("body = %s, want %s", rec.Body.String(), string(body))
			}
		})
	}
}

func TestServeGzipCheckpointETagAnd304(t *testing.T) {
	payload := []byte{0x1f, 0x8b, 0x08, 0x00}
	req1 := httptest.NewRequest(http.MethodGet, "/v1/checkpoints/63/ledger", nil)
	rec1 := httptest.NewRecorder()
	ServeGzipCheckpoint(rec1, req1, payload)
	if rec1.Code != http.StatusOK {
		t.Fatalf("status = %d, want 200", rec1.Code)
	}
	etag := rec1.Header().Get("ETag")
	if etag == "" {
		t.Fatalf("expected non-empty ETag")
	}

	req2 := httptest.NewRequest(http.MethodGet, "/v1/checkpoints/63/ledger", nil)
	req2.Header.Set("If-None-Match", etag)
	rec2 := httptest.NewRecorder()
	ServeGzipCheckpoint(rec2, req2, payload)
	if rec2.Code != http.StatusNotModified {
		t.Fatalf("status = %d, want 304", rec2.Code)
	}
}
