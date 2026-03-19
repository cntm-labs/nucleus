export function WebhooksPage() {
  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Webhooks</h1>
      <div className="bg-white rounded-xl border border-gray-200 p-6 space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">Webhook URL</label>
          <input type="url" placeholder="https://your-app.com/api/webhooks/nucleus"
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-nucleus-500 outline-none" />
        </div>
        <div>
          <label className="block text-sm font-medium mb-1">Signing Secret</label>
          <div className="flex gap-2">
            <input type="text" value="whsec_••••••••••••" readOnly
              className="flex-1 px-4 py-2 border border-gray-300 rounded-lg bg-gray-50 font-mono text-sm" />
            <button className="px-4 py-2 border border-gray-300 rounded-lg text-sm hover:bg-gray-50">Rotate</button>
          </div>
        </div>
        <div>
          <h3 className="text-sm font-medium mb-2">Event Subscriptions</h3>
          <div className="grid grid-cols-2 gap-2">
            {['user.created', 'user.updated', 'user.deleted', 'session.created', 'session.revoked', 'org.member.added'].map(event => (
              <label key={event} className="flex items-center gap-2 text-sm">
                <input type="checkbox" defaultChecked className="h-4 w-4 text-nucleus-600 rounded" />
                {event}
              </label>
            ))}
          </div>
        </div>
        <button className="px-6 py-2 bg-nucleus-600 text-white rounded-lg hover:bg-nucleus-700 font-medium">Save</button>
      </div>
    </div>
  )
}
