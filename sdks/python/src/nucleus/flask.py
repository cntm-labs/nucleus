from functools import wraps
from flask import request, g, jsonify
from .verify import verify_token
from .claims import NucleusClaims

class NucleusAuth:
    def __init__(self, app=None, secret_key=None, base_url="https://api.nucleus.dev"):
        self.base_url = base_url
        if app: self.init_app(app)

    def init_app(self, app):
        app.before_request(self._before_request)

    def _before_request(self):
        auth = request.headers.get('Authorization', '')
        if auth.startswith('Bearer '):
            try: g.nucleus_claims = verify_token(auth[7:], self.base_url)
            except: g.nucleus_claims = None
        else: g.nucleus_claims = None

    def required(self, f):
        @wraps(f)
        def decorated(*args, **kwargs):
            if not getattr(g, 'nucleus_claims', None):
                return jsonify({"error": "Unauthorized"}), 401
            return f(*args, **kwargs)
        return decorated

def current_claims() -> NucleusClaims | None:
    return getattr(g, 'nucleus_claims', None)
