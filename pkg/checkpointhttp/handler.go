package checkpointhttp

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"net/http"
	"strings"
)

const ImmutableCacheControl = "public, max-age=31536000, s-maxage=31536000, immutable"

// IsAlignedCheckpoint reports whether cp is a valid 64-ledger Stellar checkpoint.
func IsAlignedCheckpoint(cp uint32) bool {
	return cp >= 63 && (uint64(cp)+1)%64 == 0
}

// NearestCheckpoint returns the checkpoint sequence containing ledger seq.
func NearestCheckpoint(seq uint32) uint32 {
	return uint32(((uint64(seq) + 64) / 64) * 64 - 1)
}

// ComputeCheckpointETag computes a deterministic ETag ("ckpt-<sequence>-<tip_hash_prefix>") for verified checkpoint responses.
func ComputeCheckpointETag(sequence uint32, tipHashPrefix string) string {
	clean := strings.TrimPrefix(tipHashPrefix, "0x")
	if len(clean) > 16 {
		clean = clean[:16]
	}
	return fmt.Sprintf(`"ckpt-%d-%s"`, sequence, clean)
}

// ServeVerifiedCheckpoint writes verified checkpoint metadata with deterministic ETag and RFC 9110 conditional caching.
func ServeVerifiedCheckpoint(w http.ResponseWriter, r *http.Request, sequence uint32, tipHashPrefix string, body []byte) {
	etag := ComputeCheckpointETag(sequence, tipHashPrefix)
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.Header().Set("Cache-Control", ImmutableCacheControl)
	w.Header().Set("ETag", etag)

	inm := r.Header.Get("If-None-Match")
	if inm != "" && (inm == etag || inm == "*" || strings.Contains(inm, etag)) {
		w.WriteHeader(http.StatusNotModified)
		return
	}

	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(body)
}

// ServeGzipCheckpoint writes raw .xdr.gz bytes with strong ETag and immutable headers.
func ServeGzipCheckpoint(w http.ResponseWriter, r *http.Request, rawGz []byte) {
	sum := sha256.Sum256(rawGz)
	etag := fmt.Sprintf(`"%s"`, hex.EncodeToString(sum[:16]))
	w.Header().Set("Content-Type", "application/gzip")
	w.Header().Set("Cache-Control", ImmutableCacheControl)
	w.Header().Set("ETag", etag)
	if r.Header.Get("If-None-Match") == etag {
		w.WriteHeader(http.StatusNotModified)
		return
	}
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(rawGz)
}
